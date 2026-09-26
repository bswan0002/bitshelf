#![cfg(unix)]
use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output},
};

struct Fixture {
    temp: tempfile::TempDir,
    config: PathBuf,
    bin: PathBuf,
}
impl Fixture {
    fn new(aliases: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let bin = temp.path().join("bin");
        fs::create_dir(&bin).unwrap();
        fs::write(
            &config,
            format!(
                "store = '{}'\n{aliases}",
                temp.path().join("store").display()
            ),
        )
        .unwrap();
        Self { temp, config, bin }
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_bs"));
        c.args(["--config", self.config.to_str().unwrap()])
            .env("PATH", &self.bin)
            .env_remove("BS_CONFIG")
            .current_dir(self.temp.path());
        c
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
    fn json(&self, args: &[&str]) -> Value {
        let out = self.run(args);
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn executable(&self, name: &str, body: &str) -> PathBuf {
        let path = self.bin.join(format!("bs-{name}"));
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
}

#[test]
fn recipes_route_bindings_quote_values_and_forward_literals() {
    let f = Fixture::new(
        r#"
[aliases.report]
description = "Print selected label and arguments"
usage = "report [--label=VALUE] [ARGS...]"
examples = ["bs report --label=hello notes"]
run = "printf '%s\\n' {{ label }} {{ args }}"
defaults = { label = "default" }
"#,
    );
    let malicious = "a b'\"; $(touch NEVER) `touch NEVER`\nnext";
    let out = f.run(&[
        "report",
        &format!("--label={malicious}"),
        "x y",
        "--other=value",
        "--",
        "--label=literal",
    ]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("{malicious}\nx y\n--other=value\n--label=literal\n")
    );
    assert!(!f.temp.path().join("NEVER").exists());
    assert_eq!(f.run(&["report"]).stdout, b"default\n");
    let entry = f.json(&["aliases", "show", "report", "--json"]);
    assert_eq!(entry["kind"], "recipe");
    assert_eq!(entry["parameters"], serde_json::json!(["label"]));
    assert_eq!(entry["defaults"]["label"], "default");
    assert_eq!(entry["examples"][0], "bs report --label=hello notes");
}

#[test]
fn recipe_inspection_and_rendering_never_execute_shell() {
    let f = Fixture::new(
        r#"
[aliases]
risky = "printf executed > MARKER; printf '%s' {{ value }}"
"#,
    );
    for args in [
        vec!["--help"],
        vec!["risky", "--help"],
        vec!["aliases", "--json"],
        vec!["aliases", "show", "risky", "--json"],
    ] {
        assert!(f.run(&args).status.success());
    }
    assert!(!f.run(&["aliases", "dry-run", "risky"]).status.success());
    let p = f.json(&[
        "aliases",
        "dry-run",
        "risky",
        "--json",
        "--",
        "--value=$(touch NEVER)",
    ]);
    assert_eq!(p["executable"], "/bin/bash");
    assert!(p["command"].as_str().unwrap().contains("'$(touch NEVER)'"));
    assert!(!f.temp.path().join("MARKER").exists());
    assert!(!f.temp.path().join("NEVER").exists());
    assert!(!f.run(&["risky"]).status.success());
    assert!(!f.temp.path().join("MARKER").exists());
}

#[test]
fn recipes_preserve_config_executable_exit_status_and_pipeline_failures() {
    let f = Fixture::new(
        r#"
[aliases]
nested = "bs aliases show nested --json"
fail = "false | true; printf unreachable"
exit-seven = "exit 7"
paths = "printf '%s\\n' {{ config_path }} {{ store_path }} {{ bs_executable }}"
"#,
    );
    // No bs executable on PATH: the recipe's bs function still uses this executable.
    assert_eq!(f.json(&["nested"])["kind"], "recipe");
    let context = f.run(&["paths"]);
    assert!(context.status.success());
    let lines: Vec<_> = std::str::from_utf8(&context.stdout)
        .unwrap()
        .lines()
        .collect();
    assert_eq!(
        lines,
        [
            f.config.to_str().unwrap(),
            f.temp.path().join("store").to_str().unwrap(),
            env!("CARGO_BIN_EXE_bs")
        ]
    );
    let failed = f.run(&["fail"]);
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    assert_eq!(f.run(&["exit-seven"]).status.code(), Some(7));
    let injected = f.temp.path().join("env.sh");
    fs::write(&injected, "printf startup > STARTUP").unwrap();
    assert!(
        f.command()
            .env("BASH_ENV", injected)
            .arg("paths")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!f.temp.path().join("STARTUP").exists());
}

#[test]
fn executable_discovery_precedence_metadata_and_safe_inspection() {
    let f = Fixture::new("[aliases]\nsame = \"printf alias\"\n");
    let exe = f.executable(
        "plugin",
        "printf ran > MARKER; printf '%s\\n' \"$BS_CONFIG\" \"$BS_EXECUTABLE\" \"$@\"; exit 7",
    );
    f.executable("same", "exit 99");
    f.executable("list", "exit 99");
    let nonexec = f.executable("nonexec", "exit 99");
    fs::set_permissions(nonexec, fs::Permissions::from_mode(0o644)).unwrap();
    fs::write(
        f.bin.join("bs-plugin.toml"),
        "description = 'Example plugin'\nusage = 'plugin ARGS...'\nexamples = ['bs plugin hi']\n",
    )
    .unwrap();
    let entries = f.json(&["aliases", "--json"]);
    assert_eq!(entries["plugin"]["kind"], "executable");
    assert_eq!(entries["plugin"]["origin"], exe.to_str().unwrap());
    assert_eq!(entries["plugin"]["description"], "Example plugin");
    assert_eq!(entries["same"]["kind"], "recipe");
    assert!(entries.get("list").is_none());
    assert!(entries.get("nonexec").is_none());
    assert_eq!(f.run(&["same"]).stdout, b"alias");
    let p = f.json(&[
        "aliases",
        "dry-run",
        "plugin",
        "--json",
        "--",
        "--help",
        "--",
        "--config=literal",
    ]);
    assert_eq!(
        p["argv"],
        serde_json::json!(["--help", "--", "--config=literal"])
    );
    assert!(!f.temp.path().join("MARKER").exists());
    let help = f.run(&["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Example plugin"));
    assert!(!f.temp.path().join("MARKER").exists());
    let out = f.run(&["--json", "plugin", "a b", "--", "--config=literal"]);
    assert_eq!(out.status.code(), Some(7));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!(
            "{}\n{}\n--json\na b\n--\n--config=literal\n",
            f.config.display(),
            env!("CARGO_BIN_EXE_bs")
        )
    );
    assert!(f.temp.path().join("MARKER").exists());
}

#[test]
fn path_order_sidecar_errors_and_discovery_before_initialization() {
    let f = Fixture::new("");
    let first = f.executable("plugin", "printf first");
    fs::write(f.bin.join("bs-plugin.toml"), "unknown = true").unwrap();
    let second = f.temp.path().join("second");
    fs::create_dir(&second).unwrap();
    fs::copy(&first, second.join("bs-plugin")).unwrap();
    fs::write(second.join("bs-plugin"), "#!/bin/sh\nprintf second\n").unwrap();
    let out = f
        .command()
        .env("PATH", std::env::join_paths([&f.bin, &second]).unwrap())
        .arg("plugin")
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"first");
    let entry = f.json(&["aliases", "show", "plugin", "--json"]);
    assert!(
        entry["metadata_error"]
            .as_str()
            .unwrap()
            .contains("unknown")
    );
    fs::remove_file(&f.config).unwrap();
    assert_eq!(f.run(&["plugin"]).stdout, b"first");
    assert_eq!(
        f.json(&["aliases", "--json"])["plugin"]["kind"],
        "executable"
    );
    assert!(!f.config.exists());
}

#[test]
fn rich_argv_helpers_and_render_only_builtin_preview() {
    let f = Fixture::new(
        r#"
[aliases.recent]
argv = ["list", "--sort", "created", "--reverse"]
description = "Newest bits"
[aliases.archive]
argv = ["move", "{id}", "archive/{shelf}.{name}"]
[aliases.helper]
exec = ["/bin/sh", "-c", "printf executed > MARKER"]
description = "A helper"
"#,
    );
    let entry = f.json(&["aliases", "show", "recent", "--json"]);
    assert_eq!(entry["kind"], "argv");
    let p = f.json(&["aliases", "dry-run", "recent", "--json", "--", "notes"]);
    assert_eq!(
        p["argv"],
        serde_json::json!(["list", "--sort", "created", "--reverse", "notes"])
    );
    let p = f.json(&[
        "aliases",
        "dry-run",
        "archive",
        "--json",
        "--",
        "notes/hello",
    ]);
    assert_eq!(
        p["argv"],
        serde_json::json!(["move", "--", "notes/hello", "archive/notes.hello"])
    );
    // No store needed: this previews execution, not the built-in move operation.
    assert!(!f.temp.path().join("store").exists());
    f.json(&["aliases", "dry-run", "helper", "--json"]);
    assert!(!f.temp.path().join("MARKER").exists());
}

#[test]
fn malformed_definitions_are_rejected() {
    for definition in [
        "x = { run = 'echo', argv = ['list'] }",
        "x = { description = 'missing implementation' }",
        "x = { run = 'echo', typo = true }",
        "x = { argv = ['list'], defaults = { count = '1' } }",
        "x = { run = 'echo {{ count }}', defaults = { typo = '1' } }",
        "x = { run = 'echo {{ args }}', defaults = { args = '1' } }",
        "x = 'echo {{ missing'",
        "x = 'echo {{ args[0] }}'",
        "x = 'echo {{ 9x }}'",
        "x = 'echo {{ config }}'",
        "x = ''",
        "list = 'echo shadow'",
    ] {
        let f = Fixture::new(&format!("[aliases]\n{definition}\n"));
        assert!(
            !f.run(&["aliases", "--json"]).status.success(),
            "{definition}"
        );
    }
}

#[test]
fn nested_config_environment_and_explicit_overrides() {
    let f = Fixture::new("[aliases]\nchosen = 'printf first'\n");
    let other = f.temp.path().join("other.toml");
    fs::write(
        &other,
        "store = '/tmp/unused'\n[aliases]\nchosen = 'printf second'\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bs"))
        .env("BS_CONFIG", &f.config)
        .args(["chosen"])
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"first");
    let out = Command::new(env!("CARGO_BIN_EXE_bs"))
        .env("BS_CONFIG", &f.config)
        .args(["chosen", "--config", other.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.stdout, b"second");
    let preview = f.json(&[
        "aliases",
        "dry-run",
        "chosen",
        "--json",
        "--",
        "--config",
        other.to_str().unwrap(),
    ]);
    assert_eq!(preview["environment"]["BS_CONFIG"], other.to_str().unwrap());
    assert!(
        preview["command"]
            .as_str()
            .unwrap()
            .ends_with("printf second")
    );
}

#[test]
fn external_arguments_preserve_non_utf8_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let f = Fixture::new("");
    f.executable("bytes", "printf '%s' \"$1\"");
    let bytes = vec![b'a', 0xff, b'b'];
    let out = f
        .command()
        .arg("bytes")
        .arg(OsString::from_vec(bytes.clone()))
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(out.stdout, bytes);
}
