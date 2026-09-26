//! Real-process lock release and contention; all paths are temporary.
use std::{
    fs,
    io::{BufRead, Write},
    process::{Command, Stdio},
};
#[test]
fn lock_child() {
    let Some(path) = std::env::var_os("BS_TEST_LOCK_PATH") else {
        return;
    };
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    file.lock().unwrap();
    println!("LOCKED");
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}
#[test]
fn competing_writer_times_out_then_recovers_after_process_death() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let config = tmp.path().join("bs.toml");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_bs"))
            .arg("--config")
            .arg(&config)
            .args(args)
            .output()
            .unwrap()
    };
    assert!(
        run(&["init", "--store", root.to_str().unwrap()])
            .status
            .success()
    );
    fs::create_dir(root.join(".bitshelf")).unwrap();
    let path = root.join(".bitshelf/writer.lock");
    fs::write(&path, "").unwrap();
    for signal in ["-INT", "-KILL"] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "lock_child", "--nocapture"])
            .env("BS_TEST_LOCK_PATH", &path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let reader = std::io::BufReader::new(child.stdout.take().unwrap());
        let mut ready = false;
        for line in reader.lines() {
            if line.unwrap().contains("LOCKED") {
                ready = true;
                break;
            }
        }
        assert!(ready);
        let result = run(&["add", "notes/blocked", "--file", "/dev/null"]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("timed out"));
        assert!(!root.join("notes/bits/blocked.md").exists());
        assert!(
            Command::new("kill")
                .args([signal, &child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        child.wait().unwrap();
        let id = if signal == "-INT" {
            "notes/after-int"
        } else {
            "notes/after-kill"
        };
        assert!(run(&["add", id, "--file", "/dev/null"]).status.success());
    }
}
