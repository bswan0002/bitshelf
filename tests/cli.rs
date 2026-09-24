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
        fs::write(self.bit_path(id), text).unwrap();
    }
    fn bit_path(&self, id: &str) -> PathBuf {
        let (shelf, slug) = id.split_once('/').unwrap();
        if slug == "SHELF" {
            self.root.join(shelf).join("SHELF.md")
        } else {
            self.root
                .join(shelf)
                .join("bits")
                .join(format!("{slug}.md"))
        }
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
    let before = fs::read(f.root.join("notes/bits/unknown.md")).unwrap();
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
    assert_eq!(
        fs::read(f.root.join("notes/bits/unknown.md")).unwrap(),
        before
    );
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
    fs::remove_dir_all(f.root.join("ui/bits")).unwrap();
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
    assert!(f.root.join("tmp/bits/expired.md").exists());
    f.run(&["prune", "--json"]);
    assert!(!f.root.join("tmp/bits/expired.md").exists());
    for p in [
        "notes/permanent",
        "tmp/SHELF",
        "tmp/future",
        "tmp/missing",
        "tmp/invalid",
    ] {
        assert!(f.bit_path(p).exists());
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
    fs::create_dir_all(f.root.join("my shelf/bits")).unwrap();
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
        f.root.join("tmp/bits/victim.md"),
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
        assert!(lines.contains(f.root.join("notes/bits/a.md").to_str().unwrap()));
        assert!(lines.contains(f.root.join("notes/bits/b.md").to_str().unwrap()));
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
    fs::remove_file(f.root.join("notes/bits/fresh.md")).unwrap();
    assert!(!complete("show notes/").contains("notes/fresh"));
    f.ok(&["shelf", "add", "ui"]);
    assert!(complete("search x --shelf u").contains("ui"));
    assert!(complete("context ").contains("ui"));
    assert!(complete("context n").contains("notes"));
}

#[test]
fn edit_and_sync_manage_reserved_timestamps_and_preserve_body() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Tracked"]);
    let path = f.root.join("notes/bits/tracked.md");
    let before = fs::read_to_string(&path).unwrap();
    let metadata = |raw: &str| -> serde_yaml::Value {
        serde_yaml::from_str(raw.split("---").nth(1).unwrap()).unwrap()
    };
    let first = metadata(&before);
    assert_eq!(first["created"], first["updated"]);
    let body = f._temp.path().join("body.txt");
    fs::write(&body, "Exact body\r\nno final newline").unwrap();
    let result = f.json(&[
        "edit",
        "notes/tracked",
        "--file",
        body.to_str().unwrap(),
        "--tags",
        "test",
        "--json",
    ]);
    assert_eq!(result["changed"], true);
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.ends_with("Exact body\r\nno final newline"));
    let edited = metadata(&after);
    assert_eq!(edited["created"], first["created"]);
    assert_ne!(edited["updated"], first["updated"]);
    assert_eq!(
        f.json(&[
            "edit",
            "notes/tracked",
            "--file",
            body.to_str().unwrap(),
            "--tags",
            "test",
            "--json"
        ])["changed"],
        false
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), after);
    let state = f.root.join(".bitshelf/state.json");
    let state_before = fs::read(&state).unwrap();
    fs::write(&path, format!("{after}\nmanual edit")).unwrap();
    let direct = fs::read(&path).unwrap();
    let preview = f.json(&["sync", "notes", "--dry-run", "--json"]);
    assert_eq!(preview["results"][0]["changed"], true);
    assert_eq!(fs::read(&path).unwrap(), direct);
    assert_eq!(fs::read(&state).unwrap(), state_before);
    f.ok(&["list", "--sort", "updated", "--reverse", "--json"]);
    f.ok(&["show", "notes/tracked", "--json"]);
    assert_eq!(fs::read(&path).unwrap(), direct);
    assert_eq!(fs::read(&state).unwrap(), state_before);
    assert_eq!(
        f.json(&["sync", "notes", "--json"])["results"][0]["changed"],
        true
    );
    let synced = fs::read_to_string(&path).unwrap();
    assert_eq!(metadata(&synced)["created"], first["created"]);
    assert!(synced.ends_with("manual edit"));
    assert_eq!(
        f.json(&["sync", "notes", "--json"])["results"][0]["changed"],
        false
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), synced);
    // Reserved-only tampering is repaired, not counted as a content edit.
    fs::write(
        &path,
        synced.replace(
            metadata(&synced)["created"].as_str().unwrap(),
            "2000-01-01T00:00:00Z",
        ),
    )
    .unwrap();
    let result = f.json(&["sync", "--json"]);
    assert_eq!(result["results"][0]["changed"], false);
    assert_eq!(result["results"][0]["metadata_changed"], true);
    assert_eq!(
        metadata(&fs::read_to_string(&path).unwrap())["created"],
        first["created"]
    );
}

#[test]
fn sync_baselines_legacy_files_and_reports_invalid_bits() {
    let f = Fixture::new();
    f.write("notes/legacy", "---\ntitle: Legacy\ncreated: 2020-01-01T00:00:00Z\ncustom: keep\nexpires: 2099-01-01T00:00:00Z\n---\nbody");
    f.write("notes/broken", "---\ntitle: [\n---\nbroken");
    let path = f.root.join("notes/bits/legacy.md");
    let original = fs::read(&path).unwrap();
    let preview = f.run(&["sync", "--dry-run", "--json"]);
    assert!(!preview.status.success());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(!f.root.join(".bitshelf").exists());
    let out = f.run(&["sync", "--json"]);
    assert!(!out.status.success());
    let results: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(results["results"][0]["error"].is_string());
    assert_eq!(results["results"][1]["baselined"], true);
    let raw = fs::read_to_string(&path).unwrap();
    assert!(raw.contains("created: 2020-01-01T00:00:00Z"));
    assert!(raw.contains("updated:"));
    assert!(raw.contains("custom: keep"));
    assert!(raw.contains("expires: 2099-01-01T00:00:00Z"));
    assert!(raw.ends_with("body"));
    assert_eq!(
        fs::read_to_string(f.root.join("notes/bits/broken.md")).unwrap(),
        "---\ntitle: [\n---\nbroken"
    );
}

#[test]
fn chronological_sort_and_updated_validation() {
    let f = Fixture::new();
    f.write(
        "notes/a",
        "---\ncreated: 2024-01-01T00:00:00Z\nupdated: 2026-01-01T00:00:00Z\n---\na",
    );
    f.write(
        "notes/b",
        "---\ncreated: 2025-01-01T00:00:00Z\nupdated: 2025-01-01T00:00:00Z\n---\nb",
    );
    f.write("notes/c", "undated");
    let ids = |args: &[&str]| {
        f.json(args)
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ids(&["list", "--sort", "created", "--reverse", "--json"]),
        ["notes/b", "notes/a", "notes/c"]
    );
    assert_eq!(
        ids(&["list", "--sort", "updated", "--json"]),
        ["notes/b", "notes/a", "notes/c"]
    );
    assert_eq!(
        ids(&["list", "--sort", "updated", "--reverse", "--json"]),
        ["notes/a", "notes/b", "notes/c"]
    );
    f.write("notes/bad", "---\nupdated: tomorrow\n---\nbody");
    assert!(!f.run(&["validate", "notes", "--json"]).status.success());
}

#[test]
fn editor_edit_waits_and_preserves_drafts_on_failure() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Edit me"]);
    let path = f.root.join("notes/bits/edit-me.md");
    let initial = fs::read_to_string(&path).unwrap();
    let script = f._temp.path().join("editor.sh");
    fs::write(&script, "printf '\neditor content' >> \"$1\"\n").unwrap();
    fs::write(
        &f.config,
        format!(
            "store = '{}'\neditor = ['/bin/sh', '{}']\n",
            f.root.display(),
            script.display()
        ),
    )
    .unwrap();
    f.ok(&["edit", "notes/edit-me"]);
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .ends_with("editor content")
    );
    let saved = fs::read(&path).unwrap();
    fs::write(&script, "printf '\nunsaved content' >> \"$1\"\nexit 1\n").unwrap();
    let failed = f.run(&["edit", "notes/edit-me"]);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("draft preserved"));
    assert_eq!(fs::read(&path).unwrap(), saved);
    assert!(fs::read_dir(f.root.join("notes/bits")).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".edit-")
    }));
    assert!(!initial.ends_with("editor content"));
    // JSON never launches an editor implicitly.
    assert!(!f.run(&["edit", "notes/edit-me", "--json"]).status.success());
}

#[test]
fn invalid_edit_does_not_change_file_or_tracking_and_lock_is_respected() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Safe"]);
    let path = f.root.join("notes/bits/safe.md");
    let state = f.root.join(".bitshelf/state.json");
    let raw = fs::read(&path).unwrap();
    let tracked = fs::read(&state).unwrap();
    assert!(
        !f.run(&["edit", "notes/safe", "--title", "", "--json"])
            .status
            .success()
    );
    assert_eq!(fs::read(&path).unwrap(), raw);
    assert_eq!(fs::read(&state).unwrap(), tracked);
    fs::write(f.root.join(".bitshelf/state.lock"), "another process").unwrap();
    assert!(!f.run(&["sync", "--json"]).status.success());
    assert!(
        !f.run(&["edit", "notes/safe", "--title", "Changed", "--json"])
            .status
            .success()
    );
    assert_eq!(fs::read(&path).unwrap(), raw);
}

#[test]
fn stdin_edit_preserves_unknown_metadata_expiration_and_reserved_dates() {
    let f = Fixture::new();
    f.write("notes/imported", "---\ntitle: Imported\ncreated: 2020-01-01T00:00:00Z\nupdated: 2021-01-01T00:00:00Z\nexpires: 2099-01-01T00:00:00Z\ncustom: {keep: true}\n---\nold");
    let mut child = f
        .command(&["edit", "notes/imported", "--stdin", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"---\nbody, not metadata\n---\nexact")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let raw = fs::read_to_string(f.root.join("notes/bits/imported.md")).unwrap();
    assert!(raw.contains("created: 2020-01-01T00:00:00Z"));
    assert!(raw.contains("expires: 2099-01-01T00:00:00Z"));
    assert!(raw.contains("keep: true"));
    assert!(raw.ends_with("---\nbody, not metadata\n---\nexact"));
    for command in ["add", "edit"] {
        assert!(
            !f.run(&[
                command,
                "notes/imported",
                "--created",
                "2000-01-01T00:00:00Z"
            ])
            .status
            .success()
        );
        assert!(
            !f.run(&[
                command,
                "notes/imported",
                "--updated",
                "2000-01-01T00:00:00Z"
            ])
            .status
            .success()
        );
    }
}

#[test]
fn concurrent_direct_edits_are_not_overwritten_and_corrupt_state_is_not_discarded() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Concurrent"]);
    let path = f.root.join("notes/bits/concurrent.md");
    let script = f._temp.path().join("editor.sh");
    fs::write(
        &script,
        format!(
            "printf concurrent >> '{}'\nprintf draft >> \"$1\"\n",
            path.display()
        ),
    )
    .unwrap();
    fs::write(
        &f.config,
        format!(
            "store = '{}'\neditor = ['/bin/sh', '{}']\n",
            f.root.display(),
            script.display()
        ),
    )
    .unwrap();
    let output = f.run(&["edit", "notes/concurrent"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("changed during operation"));
    let raw = fs::read(&path).unwrap();
    assert!(String::from_utf8_lossy(&raw).ends_with("concurrent"));
    fs::write(f.root.join(".bitshelf/state.json"), "").unwrap();
    assert!(!f.run(&["sync", "--json"]).status.success());
    assert_eq!(fs::read(&path).unwrap(), raw);
    assert_eq!(
        fs::read_to_string(f.root.join(".bitshelf/state.json")).unwrap(),
        ""
    );
}

#[cfg(unix)]
#[test]
fn lifecycle_refuses_symlinked_bits_and_internal_state() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let outside = f._temp.path().join("outside.md");
    fs::write(&outside, "outside").unwrap();
    symlink(&outside, f.root.join("notes/bits/link.md")).unwrap();
    assert!(
        !f.run(&["edit", "notes/link", "--title", "Unsafe"])
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(&outside).unwrap(), "outside");
    let state_dir = f._temp.path().join("state");
    fs::create_dir(&state_dir).unwrap();
    symlink(&state_dir, f.root.join(".bitshelf")).unwrap();
    assert!(!f.run(&["sync", "--json"]).status.success());
    assert!(
        !f.run(&["add", "notes", "--title", "Unsafe"])
            .status
            .success()
    );
    assert!(!f.root.join("notes/bits/unsafe.md").exists());
    assert_eq!(fs::read_dir(state_dir).unwrap().count(), 0);
}

#[test]
fn sync_tracks_incomplete_bits_without_enforcing_shelf_requirements() {
    let f = Fixture::new();
    f.ok(&["shelf", "add", "ui", "--required", "title,tags"]);
    f.write("ui/abc", "unfinished note");
    f.write("ui/bad-tags", "---\ntags: not-a-list\n---\nbody");
    let preview = f.ok(&["sync", "ui", "--dry-run", "--json"]);
    assert!(preview.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(f.root.join("ui/bits/abc.md")).unwrap(),
        "unfinished note"
    );
    let synced = f.ok(&["sync", "ui", "--json"]);
    assert!(synced.stderr.is_empty());
    let value: Value = serde_json::from_slice(&synced.stdout).unwrap();
    assert!(
        value["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["baselined"] == true && r["error"].is_null())
    );
    let path = f.root.join("ui/bits/abc.md");
    let baseline = fs::read_to_string(&path).unwrap();
    assert!(baseline.contains("created:"));
    assert!(baseline.contains("updated:"));
    assert!(!baseline.contains("title:"));
    assert!(!baseline.contains("tags:"));
    assert!(baseline.ends_with("unfinished note"));
    fs::write(&path, format!("{baseline} edited")).unwrap();
    let updated = f.json(&["sync", "ui", "--json"]);
    assert_eq!(updated["results"][0]["changed"], true);
    assert!(
        fs::read_to_string(f.root.join("ui/bits/bad-tags.md"))
            .unwrap()
            .contains("tags: not-a-list")
    );
    // Authoring validation remains strict and separate from timestamp tracking.
    assert!(!f.run(&["validate", "ui", "--json"]).status.success());
    assert!(
        !f.run(&["edit", "ui/abc", "--title", "Still missing tags", "--json"])
            .status
            .success()
    );
}

#[test]
fn shelf_local_configuration_and_storage_are_portable() {
    let f = Fixture::new();
    f.ok(&[
        "shelf",
        "add",
        "docs",
        "--description",
        "Documentation",
        "--required",
        "title,tags",
    ]);
    let global = fs::read_to_string(&f.config).unwrap();
    assert!(!global.contains("shelves"));
    assert!(f.root.join("notes/bits").is_dir());
    assert!(f.root.join("notes/bs.toml").is_file());
    assert!(
        fs::read_to_string(f.root.join("docs/bs.toml"))
            .unwrap()
            .contains("Documentation")
    );
    fs::rename(f.root.join("docs"), f.root.join("renamed")).unwrap();
    let context = f.json(&["context", "renamed", "--json"]);
    assert_eq!(context["description"], "Documentation");
    assert_eq!(
        context["bits_path"],
        f.root.join("renamed/bits").to_str().unwrap()
    );
    assert_eq!(context["path"], f.root.join("renamed").to_str().unwrap());
    assert!(
        !f.run(&["add", "renamed", "--title", "No tags"])
            .status
            .success()
    );
    f.ok(&[
        "add",
        "renamed",
        "--title",
        "Tagged",
        "--tags",
        "repo,project",
    ]);
    assert!(f.root.join("renamed/bits/tagged.md").is_file());
    f.ok(&["shelf", "add", "renamed"]);
    assert_eq!(
        f.json(&["context", "renamed", "--json"])["required"],
        serde_json::json!(["title", "tags"])
    );
    fs::remove_dir_all(f.root.join("renamed")).unwrap();
    assert!(
        !f.json(&["shelf", "list", "--json"])
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["name"] == "renamed")
    );
}

#[test]
fn auxiliary_files_are_ignored_and_preserved() {
    let f = Fixture::new();
    f.ok(&["shelf", "add", "docs", "--retention", "1d"]);
    fs::create_dir_all(f.root.join("docs/scripts")).unwrap();
    fs::create_dir_all(f.root.join("unrelated")).unwrap();
    let auxiliary = [
        "docs/SHELF.md",
        "docs/README.md",
        "docs/scripts/import.py",
        "docs/scripts/example.md",
        "unrelated/readme.md",
    ];
    let content = "---\nexpires: 2000-01-01T00:00:00Z\n---\nauxiliary-needle";
    for path in auxiliary {
        fs::write(f.root.join(path), content).unwrap();
    }
    f.write("docs/expired", content);
    // SHELF is only guidance at the shelf root; inside bits it is an ordinary bit.
    fs::write(f.root.join("docs/bits/SHELF.md"), "ordinary bit").unwrap();
    assert_eq!(
        f.json(&["list", "docs", "--json"])
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        f.json(&["search", "auxiliary-needle", "--json"])
            .as_array()
            .unwrap()
            .len(),
        1
    );
    f.ok(&["sync", "--json"]);
    f.ok(&["validate", "--json"]);
    // Remove the nonexpiring bit to allow pruning to succeed without a skip warning.
    fs::remove_file(f.root.join("docs/bits/SHELF.md")).unwrap();
    f.ok(&["prune", "--json"]);
    assert!(!f.root.join("docs/bits/expired.md").exists());
    for path in auxiliary {
        assert_eq!(fs::read_to_string(f.root.join(path)).unwrap(), content);
    }
    assert!(
        !f.json(&["shelf", "list", "--json"])
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["name"] == "unrelated")
    );
}

#[test]
fn invalid_shelf_config_fails_even_when_empty() {
    let f = Fixture::new();
    for text in [
        "invalid toml [",
        "retention = '0d'",
        "required = ['unknown']",
        "typo = true",
    ] {
        fs::write(f.root.join("notes/bs.toml"), text).unwrap();
        for args in [
            vec!["context", "notes"],
            vec!["list", "notes"],
            vec!["validate"],
            vec!["shelf", "list"],
            vec!["shelf", "add", "notes"],
        ] {
            let out = f.run(&args);
            assert!(!out.status.success(), "{args:?}: {text}");
            assert!(
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                )
                .contains("bs.toml")
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn refuses_symlinked_bits_directories_and_shelf_configs() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let external = tempfile::tempdir().unwrap();
    fs::remove_dir(f.root.join("notes/bits")).unwrap();
    symlink(external.path(), f.root.join("notes/bits")).unwrap();
    for args in [
        vec!["list"],
        vec!["add", "notes", "--title", "Bad"],
        vec!["shelf", "add", "notes"],
    ] {
        assert!(!f.run(&args).status.success());
    }
    fs::remove_file(f.root.join("notes/bits")).unwrap();
    fs::create_dir(f.root.join("notes/bits")).unwrap();
    fs::write(external.path().join("bs.toml"), "required = []").unwrap();
    fs::remove_file(f.root.join("notes/bs.toml")).unwrap();
    symlink(
        external.path().join("bs.toml"),
        f.root.join("notes/bs.toml"),
    )
    .unwrap();
    for args in [
        vec!["context", "notes"],
        vec!["list"],
        vec!["shelf", "add", "notes"],
    ] {
        assert!(!f.run(&args).status.success());
    }
    assert_eq!(
        fs::read_to_string(external.path().join("bs.toml")).unwrap(),
        "required = []"
    );
}

#[test]
fn editor_does_not_hold_store_lock_and_commit_reloads_state() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Editing"]);
    let script = f._temp.path().join("editor.sh");
    fs::write(&script, format!(
        "set -eu\n'{}' --config '{}' add notes --title Parallel\n'{}' --config '{}' sync\nprintf '\\neditor content' >> \"$1\"\n",
        env!("CARGO_BIN_EXE_bs"), f.config.display(),
        env!("CARGO_BIN_EXE_bs"), f.config.display(),
    )).unwrap();
    fs::write(
        &f.config,
        format!(
            "store = '{}'\neditor = ['/bin/sh', '{}']\n",
            f.root.display(),
            script.display(),
        ),
    )
    .unwrap();
    f.ok(&["edit", "notes/editing"]);
    assert!(
        fs::read_to_string(f.bit_path("notes/editing"))
            .unwrap()
            .ends_with("editor content")
    );
    let state: Value =
        serde_json::from_slice(&fs::read(f.root.join(".bitshelf/state.json")).unwrap()).unwrap();
    assert!(state["entries"]["notes/parallel"].is_object());
    assert!(state["entries"]["notes/editing"].is_object());
    assert_eq!(
        f.json(&["sync", "--json"])["results"][1]["baselined"],
        false
    );
}

#[test]
fn sync_forgets_missing_paths_without_changing_live_history() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Deleted"]);
    f.ok(&["add", "notes", "--title", "Live"]);
    f.ok(&["shelf", "add", "removed"]);
    f.ok(&["add", "removed", "--title", "Old"]);
    let state_path = f.root.join(".bitshelf/state.json");
    let before = fs::read(&state_path).unwrap();
    let tracked: Value = serde_json::from_slice(&before).unwrap();
    fs::remove_file(f.bit_path("notes/deleted")).unwrap();
    fs::remove_dir_all(f.root.join("removed")).unwrap();
    f.ok(&["sync", "notes", "--dry-run"]);
    assert_eq!(fs::read(&state_path).unwrap(), before);
    // Scoped sync also retires records for definitively absent paths elsewhere.
    f.ok(&["sync", "notes"]);
    let state: Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    assert!(state["entries"]["notes/deleted"].is_null());
    assert!(state["entries"]["removed/old"].is_null());
    assert_eq!(
        state["entries"]["notes/live"],
        tracked["entries"]["notes/live"]
    );
    f.write(
        "notes/deleted",
        "---\ncreated: 2001-01-01T00:00:00Z\nupdated: 2002-01-01T00:00:00Z\n---\nreplacement",
    );
    let result = f.json(&["sync", "notes", "--json"]);
    assert_eq!(result["results"][0]["baselined"], true);
    assert!(
        fs::read_to_string(f.bit_path("notes/deleted"))
            .unwrap()
            .contains("created: 2001-01-01T00:00:00Z")
    );
}

#[test]
fn rename_sync_and_rename_back_starts_fresh_tracking() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Original"]);
    fs::rename(f.bit_path("notes/original"), f.bit_path("notes/moved")).unwrap();
    assert_eq!(f.json(&["sync", "--json"])["results"][0]["baselined"], true);
    fs::rename(f.bit_path("notes/moved"), f.bit_path("notes/original")).unwrap();
    assert_eq!(f.json(&["sync", "--json"])["results"][0]["baselined"], true);
    let state: Value =
        serde_json::from_slice(&fs::read(f.root.join(".bitshelf/state.json")).unwrap()).unwrap();
    assert_eq!(state["entries"].as_object().unwrap().len(), 1);
    assert!(state["entries"]["notes/original"].is_object());
}

#[test]
fn atomic_external_save_preserves_history_and_detects_changes() {
    let f = Fixture::new();
    f.ok(&["add", "notes", "--title", "Atomic"]);
    let path = f.bit_path("notes/atomic");
    let before = f.json(&["list", "--json"])[0]["metadata"].clone();
    let replacement = f._temp.path().join("replacement.md");
    fs::write(
        &replacement,
        format!("{}new body", fs::read_to_string(&path).unwrap()),
    )
    .unwrap();
    fs::rename(replacement, &path).unwrap();
    let synced = f.json(&["sync", "--json"]);
    assert_eq!(synced["results"][0]["baselined"], false);
    assert_eq!(synced["results"][0]["changed"], true);
    let after = f.json(&["list", "--json"])[0]["metadata"].clone();
    assert_eq!(after["created"], before["created"]);
    assert_ne!(after["updated"], before["updated"]);
}

#[test]
fn prune_forgets_removed_bits_and_respects_lock() {
    let f = Fixture::new();
    f.ok(&["shelf", "add", "tmp", "--retention", "1d"]);
    f.write(
        "tmp/old",
        "---\ncreated: 2000-01-01T00:00:00Z\nexpires: 2000-01-02T00:00:00Z\n---\nold",
    );
    f.ok(&["sync"]);
    let state_path = f.root.join(".bitshelf/state.json");
    let before = fs::read(&state_path).unwrap();
    let lock = f.root.join(".bitshelf/state.lock");
    fs::write(&lock, "another command").unwrap();
    f.ok(&["prune", "--dry-run"]);
    assert!(!f.run(&["prune"]).status.success());
    assert!(f.bit_path("tmp/old").exists());
    assert_eq!(fs::read(&state_path).unwrap(), before);
    fs::remove_file(lock).unwrap();
    f.ok(&["prune"]);
    let state: Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    assert!(state["entries"]["tmp/old"].is_null());
    f.write("tmp/old", "replacement");
    assert_eq!(f.json(&["sync", "--json"])["results"][0]["baselined"], true);
}

#[cfg(unix)]
#[test]
fn validation_reports_links_without_following_them_and_checks_other_bits() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("victim.md"), "outside").unwrap();
    symlink(outside.path(), f.root.join("linked-shelf")).unwrap();
    symlink("missing", f.root.join("dangling-shelf")).unwrap();
    symlink(
        outside.path().join("victim.md"),
        f.bit_path("notes/external"),
    )
    .unwrap();
    f.write("notes/regular", "regular");
    f.write("notes/invalid", "---\ntitle: []\n---\nbad");
    symlink("regular.md", f.bit_path("notes/internal")).unwrap();
    symlink("missing", f.bit_path("notes/dangling")).unwrap();
    symlink("bits/regular.md", f.root.join("notes/SHELF.md")).unwrap();
    for (shelf, field) in [("linked-bits", "bits"), ("linked-config", "bs.toml")] {
        f.ok(&["shelf", "add", shelf]);
        let path = f.root.join(shelf).join(field);
        if field == "bits" {
            fs::remove_dir(&path).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
        symlink("missing", path).unwrap();
    }
    // Helper symlinks are unmanaged, not traversed or rejected.
    fs::create_dir(f.root.join("notes/scripts")).unwrap();
    symlink(outside.path(), f.root.join("notes/scripts/helper")).unwrap();
    let out = f.run(&["validate", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let rows: Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = rows.as_array().unwrap();
    assert_eq!(
        rows.iter()
            .filter(|r| r["errors"].to_string().contains("refusing symlink"))
            .count(),
        8
    );
    assert!(
        rows.iter()
            .any(|r| r["id"] == "notes/regular" && r["valid"] == true)
    );
    assert!(
        rows.iter()
            .any(|r| r["id"] == "notes/invalid" && r["valid"] == false)
    );
    assert_eq!(
        fs::read_to_string(outside.path().join("victim.md")).unwrap(),
        "outside"
    );
    let scoped = f.run(&["validate", "notes", "--json"]);
    assert_eq!(scoped.status.code(), Some(1));
    let explicit = f.run(&["validate", "linked-shelf", "--json"]);
    assert_eq!(explicit.status.code(), Some(1));
    let rows: Value = serde_json::from_slice(&explicit.stdout).unwrap();
    assert!(
        rows[0]["errors"][0]
            .as_str()
            .unwrap()
            .contains("refusing symlink")
    );
}

#[cfg(unix)]
#[test]
fn explicit_symlink_store_root_is_supported() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let root_link = f._temp.path().join("root-link");
    symlink(&f.root, &root_link).unwrap();
    fs::write(
        &f.config,
        format!("store = '{}'\neditor = ['true']\n", root_link.display()),
    )
    .unwrap();
    f.ok(&["add", "notes", "--title", "Safe"]);
    f.ok(&["sync"]);
    f.ok(&["edit", "notes/safe", "--title", "Updated"]);
    f.ok(&["validate"]);
}

#[test]
fn lifecycle_commands_do_not_recreate_a_missing_store() {
    let f = Fixture::new();
    fs::remove_dir_all(&f.root).unwrap();
    for args in [vec!["prune"], vec!["sync"], vec!["sync", "--dry-run"]] {
        assert!(!f.run(&args).status.success());
        assert!(!f.root.exists());
    }
}

#[test]
fn prune_during_editor_session_cannot_resurrect_a_bit() {
    let f = Fixture::new();
    f.ok(&["shelf", "add", "tmp", "--retention", "1d"]);
    f.write(
        "tmp/expired",
        "---\nexpires: 2000-01-01T00:00:00Z\n---\nold",
    );
    f.ok(&["sync"]);
    let script = f._temp.path().join("editor.sh");
    fs::write(
        &script,
        format!(
            "set -eu\n'{}' --config '{}' prune tmp\nprintf '\\nrecover me' >> \"$1\"\n",
            env!("CARGO_BIN_EXE_bs"),
            f.config.display(),
        ),
    )
    .unwrap();
    fs::write(
        &f.config,
        format!(
            "store = '{}'\neditor = ['/bin/sh', '{}']\n",
            f.root.display(),
            script.display(),
        ),
    )
    .unwrap();
    let out = f.run(&["edit", "tmp/expired"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("draft preserved at"));
    assert!(!f.bit_path("tmp/expired").exists());
    let state: Value =
        serde_json::from_slice(&fs::read(f.root.join(".bitshelf/state.json")).unwrap()).unwrap();
    assert!(state["entries"]["tmp/expired"].is_null());
    let drafts: Vec<_> = fs::read_dir(f.root.join("tmp/bits"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(drafts.len(), 1);
    assert!(
        fs::read_to_string(&drafts[0])
            .unwrap()
            .ends_with("recover me")
    );
}
