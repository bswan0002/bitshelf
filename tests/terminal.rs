//! Exercise cursor rendering and editor behavior in a real controlling terminal.
#![cfg(unix)]
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};

struct Terminal {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    receiver: mpsc::Receiver<Vec<u8>>,
    parser: vt100::Parser,
}
impl Terminal {
    fn spawn(config: &Path, args: &[&str], env: &[(&str, &str)]) -> Self {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 40,
                cols: 160,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_bs"));
        cmd.arg("--config");
        cmd.arg(config);
        cmd.args(args);
        cmd.env_remove("VISUAL");
        cmd.env_remove("EDITOR");
        cmd.env("TERM", "xterm-256color");
        for (key, value) in env {
            cmd.env(key, value);
        }
        let child = pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buffer = [0; 8192];
            while let Ok(size) = reader.read(&mut buffer) {
                if size == 0 || sender.send(buffer[..size].to_vec()).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            writer,
            receiver,
            parser: vt100::Parser::new(40, 160, 100),
        }
    }
    fn send(&mut self, text: &str) {
        self.writer.write_all(text.as_bytes()).unwrap();
        self.writer.flush().unwrap();
    }
    fn screen(&self) -> String {
        self.parser.screen().contents()
    }
    fn wait_for(&mut self, text: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !self.screen().contains(text) {
            assert!(
                Instant::now() < deadline,
                "timeout awaiting {text:?}: {}",
                self.screen()
            );
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(bytes) => self.parser.process(&bytes),
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(e) => panic!("PTY closed awaiting {text:?}: {e}: {}", self.screen()),
            }
        }
    }
    fn finish(&mut self) -> u32 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            while let Ok(bytes) = self.receiver.try_recv() {
                self.parser.process(&bytes);
            }
            if let Some(status) = self.child.try_wait().unwrap() {
                // Drain the reader after process exit; the final frame may still be queued.
                while let Ok(bytes) = self.receiver.recv_timeout(Duration::from_millis(100)) {
                    self.parser.process(&bytes);
                }
                return status.exit_code();
            }
            assert!(
                Instant::now() < deadline,
                "PTY process timed out: {}",
                self.screen()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn init_renders_each_completed_prompt_once() {
    let tmp = tempfile::tempdir().unwrap();
    let config = tmp.path().join("config.toml");
    let mut terminal = Terminal::spawn(&config, &["init"], &[]);
    terminal.wait_for("Store directory");
    terminal.send("store\r");
    terminal.wait_for("Editor command");
    terminal.send("code\r");
    assert_eq!(terminal.finish(), 0);
    let screen = terminal.screen();
    assert_eq!(screen.matches("Store directory").count(), 1, "{screen}");
    assert_eq!(screen.matches("Editor command").count(), 1, "{screen}");
    assert!(screen.contains("Initialized"), "{screen}");
    assert!(tmp.path().join("store/notes").is_dir());
}

#[test]
fn init_uses_environment_editor_without_prompting_or_pinning_it() {
    for variable in ["VISUAL", "EDITOR"] {
        let tmp = tempfile::tempdir().unwrap();
        let config = tmp.path().join("config.toml");
        let mut terminal = Terminal::spawn(&config, &["init"], &[(variable, "vim")]);
        terminal.wait_for("Store directory");
        terminal.send("store\r");
        assert_eq!(terminal.finish(), 0);
        let screen = terminal.screen();
        assert!(!screen.contains("Editor command"), "{screen}");
        assert_eq!(screen.matches("Store directory").count(), 1, "{screen}");
        assert!(!fs::read_to_string(config).unwrap().contains("editor ="));
    }
}

#[test]
fn gui_draft_gets_wait_flag_but_open_does_not() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let config = tmp.path().join("config.toml");
    let store = tmp.path().join("store");
    fs::create_dir_all(store.join("notes")).unwrap();
    let editor = tmp.path().join("code");
    let log = tmp.path().join("args");
    fs::write(
        &editor,
        format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n", log.display()),
    )
    .unwrap();
    fs::set_permissions(&editor, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        &config,
        format!(
            "store = '{}'\neditor = ['{}', '--reuse-window']\n",
            store.display(),
            editor.display()
        ),
    )
    .unwrap();
    let mut terminal = Terminal::spawn(
        &config,
        &[
            "add",
            "notes",
            "--interactive",
            "--title",
            "Draft",
            "--tags",
            "test",
        ],
        &[],
    );
    assert_eq!(terminal.finish(), 0, "{}", terminal.screen());
    assert!(
        fs::read_to_string(&log)
            .unwrap()
            .starts_with("--wait\n--reuse-window\n")
    );
    assert!(store.join("notes/draft.md").exists());
    let mut terminal = Terminal::spawn(&config, &["open", "notes/draft"], &[]);
    assert_eq!(terminal.finish(), 0);
    assert!(
        fs::read_to_string(&log)
            .unwrap()
            .starts_with("--reuse-window\n")
    );
}
