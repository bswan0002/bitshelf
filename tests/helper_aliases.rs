//! Global helper aliases are explicit argv execution, never implicit shelf hooks.
#![cfg(unix)]
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct Fixture {
    temp: tempfile::TempDir,
    config: PathBuf,
    log: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let log = temp.path().join("log");
        let script = temp.path().join("helper.sh");
        fs::write(&script, "log=$1; shift\nprintf '%s\\n' \"$BS_CONFIG\" \"$BS_EXECUTABLE\" \"$@\" > \"$log\"\nprintf '{\"helper\":true}\\n'\n").unwrap();
        fs::write(&config, format!("store = '{}'\n[aliases]\ntickets = {{ exec = ['/bin/sh', '{}', '{}', '{{literal}}', 'a b', '$(touch NEVER)'] }}\n", temp.path().join("store").display(), script.display(), log.display())).unwrap();
        Self { temp, config, log }
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_bs"))
            .arg("--config")
            .arg(&self.config)
            .args(args)
            .current_dir(self.temp.path())
            .output()
            .unwrap()
    }
    fn logged(&self) -> Vec<String> {
        fs::read_to_string(&self.log)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

#[test]
fn helper_arguments_environment_and_global_json_are_preserved() {
    let f = Fixture::new();
    for args in [
        vec!["--json", "tickets", "--help"],
        vec!["tickets", "--json", "--help"],
    ] {
        let output = f.run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["helper"],
            true
        );
        assert_eq!(
            f.logged(),
            [
                f.config.to_str().unwrap(),
                env!("CARGO_BIN_EXE_bs"),
                "{literal}",
                "a b",
                "$(touch NEVER)",
                "--json",
                "--help"
            ]
        );
        assert!(!f.temp.path().join("NEVER").exists());
    }
}

#[test]
fn config_after_alias_is_consumed_but_literal_arguments_are_not() {
    let f = Fixture::new();
    for config_args in [
        vec!["--config".to_owned(), f.config.to_str().unwrap().to_owned()],
        vec![format!("--config={}", f.config.display())],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bs"))
            .arg("tickets")
            .args(config_args)
            .args(["--json", "--", "--config", "literal", "--config=literal"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            &f.logged()[5..],
            ["--json", "--", "--config", "literal", "--config=literal"]
        );
        assert_eq!(f.logged()[0], f.config.to_str().unwrap());
    }
}

#[test]
fn inspection_never_executes_helpers_and_aliases_have_structured_json() {
    let f = Fixture::new();
    let output = f.run(&["aliases", "--json"]);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["tickets"]["exec"][0], "/bin/sh");
    assert!(!f.log.exists());
    let human = f.run(&["aliases"]);
    assert!(human.status.success());
    let roundtrip: toml::Value =
        toml::from_str(std::str::from_utf8(&human.stdout).unwrap()).unwrap();
    assert!(roundtrip["tickets"]["exec"].is_array());
    assert!(!f.log.exists());
}

#[test]
fn malformed_helpers_and_builtin_shadowing_are_rejected() {
    let f = Fixture::new();
    for definition in [
        "tickets = { exec = [] }",
        "tickets = { exec = [''] }",
        "tickets = { exec = ['  '] }",
        "tickets = { exec = 'echo' }",
        "tickets = { exec = ['echo'], typo = true }",
        "list = { exec = ['echo'] }",
    ] {
        fs::write(
            &f.config,
            format!("store = '/tmp/unused'\n[aliases]\n{definition}\n"),
        )
        .unwrap();
        assert!(
            !f.run(&["aliases", "--json"]).status.success(),
            "{definition}"
        );
    }
}

#[test]
fn helper_exit_status_and_spawn_errors_are_preserved() {
    let f = Fixture::new();
    fs::write(&f.config, "store = '/tmp/unused'\n[aliases]\nfail = { exec = ['/bin/sh', '-c', 'exit 7'] }\nmissing = { exec = ['/nonexistent/bitshelf-helper'] }\n").unwrap();
    assert_eq!(f.run(&["fail"]).status.code(), Some(7));
    let output = f.run(&["missing"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cannot execute helper alias missing")
    );
}
