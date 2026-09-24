use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};
struct Fixture {
    _temp: tempfile::TempDir,
    config: PathBuf,
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config.toml");
        let root = temp.path().join("store");
        let f = Self {
            _temp: temp,
            config,
            root,
        };
        f.ok(&[
            "init",
            "--store",
            f.root.to_str().unwrap(),
            "--editor",
            "true",
            "--json",
        ]);
        f
    }
    fn command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_bs"));
        c.arg("--config")
            .arg(&self.config)
            .args(args)
            .stdin(Stdio::null());
        c
    }
    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }
    fn ok(&self, args: &[&str]) -> Output {
        let o = self.run(args);
        assert!(
            o.status.success(),
            "{:?}: {}",
            args,
            String::from_utf8_lossy(&o.stderr)
        );
        o
    }
    fn json(&self, args: &[&str]) -> Value {
        serde_json::from_slice(&self.ok(args).stdout).unwrap()
    }
    fn write(&self, id: &str, text: &str) {
        fs::write(self.root.join(format!("{id}.md")), text).unwrap();
    }
}
#[test]
fn verbatim_workflow_and_collision() {
    let f = Fixture::new();
    let body = "\nExact prompt\r\n```tsx\n  x();\n```\n---\nno final newline";
    let mut child = f
        .command(&[
            "add",
            "notes",
            "--title",
            "Exact Prompt",
            "--stdin",
            "--tags",
            "prompt,test",
            "--json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["id"], "notes/exact-prompt");
    let shown = f.json(&["show", "notes/exact-prompt", "--json"]);
    assert!(shown["content"].as_str().unwrap().ends_with(body));
    assert_eq!(
        f.json(&["search", "EXACT PROMPT", "--json"])
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        f.json(&["list", "notes", "--tag", "test", "--json"])
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        f.json(&["search", "not present", "--json"]),
        serde_json::json!([])
    );
    assert_eq!(
        f.run(&["add", "notes", "--title", "Exact Prompt"])
            .status
            .code(),
        Some(1)
    );
    f.ok(&[
        "add",
        "notes",
        "--title",
        "Exact Prompt",
        "--slug",
        "another",
    ]);
    f.ok(&["validate", "--json"]);
}
#[test]
fn malformed_metadata_is_readable_and_unknown_fields_survive() {
    let f = Fixture::new();
    f.write("notes/broken", "---\ntitle: [\n---\nfind me");
    f.write("notes/plain", "handwritten");
    f.write("notes/unknown", "---\ncustom: {nested: yes}\n---\nbody");
    let before = fs::read(f.root.join("notes/unknown.md")).unwrap();
    let list = f.json(&["list", "--json"]);
    assert_eq!(list.as_array().unwrap().len(), 3);
    assert!(!list[0]["errors"].as_array().unwrap().is_empty());
    assert_eq!(
        f.ok(&["show", "notes/broken"]).stdout,
        b"---\ntitle: [\n---\nfind me"
    );
    assert_eq!(
        f.json(&["search", "find me", "--json"])
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let out = f.run(&["validate", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    serde_json::from_slice::<Value>(&out.stdout).unwrap();
    assert_eq!(fs::read(f.root.join("notes/unknown.md")).unwrap(), before);
}
#[test]
fn guidance_requirements_and_missing_shelves() {
    let f = Fixture::new();
    f.ok(&[
        "shelf",
        "add",
        "ui",
        "--description",
        "Components",
        "--required",
        "title,tags",
    ]);
    fs::write(f.root.join("ui/SHELF.md"), "# UI\nFull guidance\n").unwrap();
    let context = f.json(&["context", "ui", "--json"]);
    assert_eq!(context["guidance"]["text"], "# UI\nFull guidance\n");
    assert_eq!(context["required"], serde_json::json!(["title", "tags"]));
    assert_eq!(f.json(&["list", "ui", "--json"]), serde_json::json!([]));
    f.ok(&["validate", "ui"]);
    assert!(
        !f.run(&["add", "ui", "--title", "Missing tags"])
            .status
            .success()
    );
    f.ok(&["add", "ui", "--title", "Valid", "--tags", "react"]);
    fs::remove_dir_all(f.root.join("ui")).unwrap();
    assert!(
        f.json(&["shelf", "list", "--json"])
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["name"] == "ui" && v["missing"] == true)
    );
    assert!(!f.run(&["validate"]).status.success());
    f.ok(&["shelf", "add", "ui"]);
}
#[test]
fn expiration_is_opt_in_explicit_and_safe() {
    let f = Fixture::new();
    f.ok(&["shelf", "add", "tmp", "--retention", "14d"]);
    let old = "---\nexpires: '2000-01-01T00:00:00Z'\n---\nbody";
    f.write("notes/permanent", old);
    f.write("tmp/expired", old);
    f.write("tmp/SHELF", old);
    f.write(
        "tmp/future",
        "---\nexpires: '2999-01-01T00:00:00Z'\n---\nbody",
    );
    f.write("tmp/missing", "no expiration");
    f.write("tmp/invalid", "---\nexpires: yesterday\n---\nbody");
    let dry = f.run(&["prune", "--dry-run", "--json"]);
    assert_eq!(dry.status.code(), Some(1));
    let rows: Value = serde_json::from_slice(&dry.stdout).unwrap();
    assert_eq!(
        rows.as_array()
            .unwrap()
            .iter()
            .filter(|r| r["status"] == "would_remove")
            .count(),
        1
    );
    assert!(f.root.join("tmp/expired.md").exists());
    f.run(&["prune", "--json"]);
    assert!(!f.root.join("tmp/expired.md").exists());
    for p in [
        "notes/permanent",
        "tmp/SHELF",
        "tmp/future",
        "tmp/missing",
        "tmp/invalid",
    ] {
        assert!(f.root.join(format!("{p}.md")).exists());
    }
    f.ok(&["add", "tmp", "--title", "New"]);
    let rows = f.json(&["list", "tmp", "--json"]);
    let new = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "tmp/new")
        .unwrap();
    let created =
        chrono::DateTime::parse_from_rfc3339(new["metadata"]["created"].as_str().unwrap()).unwrap();
    let expires =
        chrono::DateTime::parse_from_rfc3339(new["metadata"]["expires"].as_str().unwrap()).unwrap();
    assert_eq!((expires - created).num_days(), 14);
}
#[test]
fn usage_errors_and_config_errors_have_distinct_statuses() {
    let f = Fixture::new();
    for args in [
        vec!["add"],
        vec!["add", "notes", "--title", "x", "--stdin", "--file", "x"],
        vec!["shelf", "add"],
        vec!["unknown"],
    ] {
        assert_eq!(f.run(&args).status.code(), Some(2), "{args:?}");
    }
    assert_eq!(
        f.run(&["add", "notes", "--title", "escape", "--slug", "../evil"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(f.run(&["show", "notes/../config"]).status.code(), Some(1));
    fs::write(&f.config, "not toml = [").unwrap();
    assert_eq!(f.run(&["list"]).status.code(), Some(1));
}
#[test]
fn relative_config_paths_and_manual_names() {
    let f = Fixture::new();
    fs::write(&f.config, "store = 'store'\neditor = ['true']\n").unwrap();
    fs::create_dir(f.root.join("my shelf")).unwrap();
    f.write("my shelf/a.b note", "manual");
    f.ok(&["show", "my shelf/a.b note"]);
    f.ok(&["open", "my shelf/a.b note"]);
    assert_eq!(f.json(&["list", "--json"])[0]["id"], "my shelf/a.b note");
}
#[cfg(unix)]
#[test]
fn refuses_symlink_writes_reads_and_pruning() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let external = tempfile::tempdir().unwrap();
    fs::write(
        external.path().join("victim.md"),
        "---\nexpires: '2000-01-01T00:00:00Z'\n---\nbody",
    )
    .unwrap();
    symlink(external.path(), f.root.join("escape")).unwrap();
    assert!(!f.run(&["add", "escape", "--title", "Bad"]).status.success());
    assert!(!f.run(&["shelf", "add", "escape"]).status.success());
    f.ok(&["shelf", "add", "tmp", "--retention", "1d"]);
    symlink(
        external.path().join("victim.md"),
        f.root.join("tmp/victim.md"),
    )
    .unwrap();
    assert!(!f.run(&["show", "tmp/victim"]).status.success());
    f.ok(&["prune", "tmp"]);
    assert!(external.path().join("victim.md").exists());
    symlink(
        external.path().join("victim.md"),
        f.root.join("tmp/SHELF.md"),
    )
    .unwrap();
    assert!(!f.run(&["context", "tmp"]).status.success());
}
#[cfg(unix)]
#[test]
fn editor_arguments_and_json_are_not_shell_evaluated() {
    let f = Fixture::new();
    let log = f._temp.path().join("editor-log");
    let script = f._temp.path().join("editor.sh");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\necho editor-output\n",
            log.display()
        ),
    )
    .unwrap();
    f.write("notes/a", "body");
    f.write("notes/b", "body");
    for editor in [
        format!(
            "['/bin/sh', '{}', 'a b', '$(touch NEVER)']",
            script.display()
        ),
        format!(r#""/bin/sh '{}' 'a b' '$(touch NEVER)'""#, script.display()),
    ] {
        let config = format!("store = '{}'\neditor = {editor}\n", f.root.display());
        fs::write(&f.config, config).unwrap();
        let out = f.ok(&["open", "notes/a", "notes/b", "--json"]);
        let value: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(value["opened"], true);
        let lines = fs::read_to_string(&log).unwrap();
        assert!(lines.starts_with("a b\n$(touch NEVER)\n"));
        assert!(lines.contains(f.root.join("notes/a.md").to_str().unwrap()));
        assert!(lines.contains(f.root.join("notes/b.md").to_str().unwrap()));
    }
}
#[test]
fn dynamic_completion_uses_current_store_and_override() {
    let f = Fixture::new();
    let complete = |suffix: &str| {
        let line = format!("bs --config '{}' {suffix}", f.config.display());
        let out = Command::new(env!("CARGO_BIN_EXE_bs"))
            .args(["__complete_word__", "--shell", "fish", "--line", &line])
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap()
    };
    assert!(complete("add n").contains("notes"));
    f.write("notes/fresh", "body");
    f.write("notes/SHELF", "guidance");
    assert!(complete("open notes/").contains("notes/fresh"));
    assert!(!complete("open notes/").contains("SHELF"));
    fs::remove_file(f.root.join("notes/fresh.md")).unwrap();
    assert!(!complete("show notes/").contains("notes/fresh"));
    f.ok(&["shelf", "add", "ui"]);
    assert!(complete("search x --shelf u").contains("ui"));
}
