use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bs"))
        .arg("completion")
        .args(args)
        .env("HOME", home)
        .env("SHELL", "/bin/zsh")
        .env_remove("ZDOTDIR")
        .env_remove("XDG_CONFIG_HOME")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}
fn ok(out: Output) -> String {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn preview_confirmation_idempotence_and_uninstall() {
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".zshrc");
    fs::write(&path, "# my config\nexport KEEP=1\n").unwrap();
    let initial = fs::read(&path).unwrap();
    let preview = ok(run(home.path(), &["install", "--dry-run"]));
    assert!(preview.contains("compinit"));
    assert_eq!(fs::read(&path).unwrap(), initial);
    assert!(!run(home.path(), &["install"]).status.success());
    assert_eq!(fs::read(&path).unwrap(), initial);
    ok(run(home.path(), &["install", "--yes"]));
    let installed = fs::read(&path).unwrap();
    assert!(ok(run(home.path(), &["install", "--yes"])).contains("Already configured"));
    assert_eq!(fs::read(&path).unwrap(), installed);
    ok(run(home.path(), &["uninstall", "--dry-run"]));
    assert_eq!(fs::read(&path).unwrap(), installed);
    ok(run(home.path(), &["uninstall", "--yes"]));
    assert_eq!(fs::read(&path).unwrap(), initial);
    ok(run(home.path(), &["uninstall", "--yes"]));
    assert!(!home.path().join(".config/bitshelf").exists());
}

#[test]
fn bash_missing_file_and_no_final_newline() {
    let home = tempfile::tempdir().unwrap();
    ok(run(
        home.path(),
        &["install", "--shell", "bash", "--dry-run"],
    ));
    let path = home.path().join(".bashrc");
    assert!(!path.exists());
    ok(run(home.path(), &["install", "--shell", "bash", "--yes"]));
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("eval \"$(bs completion --shell bash)\"")
    );
    fs::write(&path, "export KEEP=1").unwrap();
    ok(run(home.path(), &["install", "--shell", "bash", "--yes"]));
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .starts_with("export KEEP=1\n# >>>")
    );
}

#[test]
fn respects_zdotdir_and_xdg() {
    let home = tempfile::tempdir().unwrap();
    for (shell, var, relative) in [
        ("zsh", "ZDOTDIR", ".zshrc"),
        ("fish", "XDG_CONFIG_HOME", "fish/completions/bs.fish"),
    ] {
        let dir = home.path().join(shell);
        let command = |action: &str| {
            Command::new(env!("CARGO_BIN_EXE_bs"))
                .args(["completion", action, "--shell", shell, "--yes"])
                .env("HOME", home.path())
                .env(var, &dir)
                .output()
                .unwrap()
        };
        ok(command("install"));
        let path = dir.join(relative);
        let installed = fs::read(&path).unwrap();
        ok(command("install"));
        assert_eq!(fs::read(&path).unwrap(), installed);
        ok(command("uninstall"));
        if shell == "fish" {
            assert!(!path.exists());
        } else {
            assert_eq!(fs::read_to_string(path).unwrap(), "");
        }
    }
    assert!(!home.path().join(".zshrc").exists());
}

#[test]
fn refuses_unmanaged_fish_and_broken_markers() {
    let home = tempfile::tempdir().unwrap();
    let fish = home.path().join(".config/fish/completions/bs.fish");
    fs::create_dir_all(fish.parent().unwrap()).unwrap();
    fs::write(&fish, "# custom completion\n").unwrap();
    assert!(
        !run(home.path(), &["install", "--shell", "fish", "--yes"])
            .status
            .success()
    );
    ok(run(home.path(), &["uninstall", "--shell", "fish", "--yes"]));
    assert_eq!(fs::read_to_string(fish).unwrap(), "# custom completion\n");
    let zsh = home.path().join(".zshrc");
    let broken = "# >>> bitshelf completion >>>\nuser content\n";
    fs::write(&zsh, broken).unwrap();
    for action in ["install", "uninstall"] {
        assert!(!run(home.path(), &[action, "--yes"]).status.success());
        assert_eq!(fs::read_to_string(&zsh).unwrap(), broken);
    }
}

#[test]
fn recognizes_manual_setup_without_claiming_ownership() {
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".zshrc");
    let manual = "eval \"$(bs completion --shell zsh)\"\n";
    fs::write(&path, manual).unwrap();
    assert!(ok(run(home.path(), &["install", "--yes"])).contains("Already configured"));
    ok(run(home.path(), &["uninstall", "--yes"]));
    assert_eq!(fs::read_to_string(path).unwrap(), manual);
}

#[test]
fn preserves_generator_and_rejects_invalid_modes() {
    let home = tempfile::tempdir().unwrap();
    for shell in ["bash", "zsh", "fish", "elvish", "nu", "powershell"] {
        assert!(!ok(run(home.path(), &["--shell", shell])).is_empty());
    }
    for args in [
        vec![],
        vec!["--shell", "zsh", "--yes"],
        vec!["install", "--shell", "nu", "--yes"],
        vec!["install", "--yes", "--json"],
    ] {
        assert!(!run(home.path(), &args).status.success());
    }
    assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn follows_symlinks_and_preserves_permissions() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".zshrc");
    let target = home.path().join("actual");
    fs::write(&target, "# user config\n").unwrap();
    symlink(&target, &path).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
    let preview = ok(run(home.path(), &["install", "--dry-run"]));
    assert!(preview.contains(" -> "));
    assert!(preview.contains(fs::canonicalize(&target).unwrap().to_str().unwrap()));
    assert_eq!(fs::read_to_string(&target).unwrap(), "# user config\n");
    ok(run(home.path(), &["install", "--yes"]));
    assert_eq!(fs::read_link(&path).unwrap(), target);
    assert!(
        fs::read_to_string(&target)
            .unwrap()
            .contains("bs completion --shell zsh")
    );
    ok(run(home.path(), &["uninstall", "--yes"]));
    assert_eq!(fs::read_link(&path).unwrap(), target);
    assert_eq!(fs::read_to_string(&target).unwrap(), "# user config\n");
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn installed_setup_registers_completion_in_available_shells() {
    let home = tempfile::tempdir().unwrap();
    let binary_dir = Path::new(env!("CARGO_BIN_EXE_bs")).parent().unwrap();
    let mut paths = vec![binary_dir.to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(paths).unwrap();
    for (shell, script) in [
        ("bash", "source \"$HOME/.bashrc\"; complete -p bs"),
        ("zsh", "source \"$HOME/.zshrc\"; [[ ${_comps[bs]} == _bs ]]"),
    ] {
        match Command::new(shell).arg("--version").output() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            result => {
                result.unwrap();
            }
        }
        ok(run(home.path(), &["install", "--shell", shell, "--yes"]));
        ok(Command::new(shell)
            .args(["-f", "-c", script])
            .env("HOME", home.path())
            .env("ZDOTDIR", home.path())
            .env("PATH", &path)
            .env_remove("BASH_ENV")
            .stdin(Stdio::null())
            .output()
            .unwrap());
    }
}

#[cfg(unix)]
#[test]
fn relative_chained_dotfile_links_survive_install_and_uninstall() {
    use std::os::unix::fs::symlink;
    for shell in ["bash", "zsh"] {
        let home = tempfile::tempdir().unwrap();
        let dotfiles = home.path().join("dotfiles");
        fs::create_dir(&dotfiles).unwrap();
        let target = dotfiles.join("actual");
        fs::write(&target, "# keep me\n").unwrap();
        symlink("actual", dotfiles.join("rc")).unwrap();
        let rc = home.path().join(format!(".{shell}rc"));
        symlink("dotfiles/rc", &rc).unwrap();
        ok(run(home.path(), &["install", "--shell", shell, "--yes"]));
        let installed = fs::read(&target).unwrap();
        assert!(
            ok(run(home.path(), &["install", "--shell", shell, "--yes"]))
                .contains("Already configured")
        );
        assert_eq!(fs::read(&target).unwrap(), installed);
        ok(run(
            home.path(),
            &["uninstall", "--shell", shell, "--dry-run"],
        ));
        assert_eq!(fs::read(&target).unwrap(), installed);
        ok(run(home.path(), &["uninstall", "--shell", shell, "--yes"]));
        assert_eq!(fs::read_link(&rc).unwrap(), Path::new("dotfiles/rc"));
        assert_eq!(
            fs::read_link(dotfiles.join("rc")).unwrap(),
            Path::new("actual")
        );
        assert_eq!(fs::read_to_string(&target).unwrap(), "# keep me\n");
    }
}

#[cfg(unix)]
#[test]
fn linked_fish_file_can_be_uninstalled_and_reinstalled_without_breaking_link() {
    use std::os::unix::fs::symlink;
    let home = tempfile::tempdir().unwrap();
    ok(run(home.path(), &["install", "--shell", "fish", "--yes"]));
    let path = home.path().join(".config/fish/completions/bs.fish");
    let target = home.path().join("dotfile.fish");
    fs::rename(&path, &target).unwrap();
    symlink(&target, &path).unwrap();
    for _ in 0..2 {
        ok(run(home.path(), &["uninstall", "--shell", "fish", "--yes"]));
        assert_eq!(fs::read_link(&path).unwrap(), target);
        assert_eq!(fs::read_to_string(&target).unwrap(), "");
        ok(run(home.path(), &["install", "--shell", "fish", "--yes"]));
        assert!(
            fs::read_to_string(&target)
                .unwrap()
                .contains("# >>> bitshelf completion >>>")
        );
        assert_eq!(fs::read_link(&path).unwrap(), target);
    }
}

#[cfg(unix)]
#[test]
fn broken_links_cycles_and_nonregular_targets_fail_without_modification() {
    use std::os::unix::fs::symlink;
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".zshrc");
    for target in ["missing", ".zshrc", "directory"] {
        if target == "directory" {
            fs::create_dir(home.path().join(target)).unwrap();
        }
        symlink(target, &path).unwrap();
        for action in ["install", "uninstall"] {
            assert!(!run(home.path(), &[action, "--yes"]).status.success());
            assert_eq!(fs::read_link(&path).unwrap(), Path::new(target));
        }
        fs::remove_file(&path).unwrap();
    }
    assert!(!home.path().join("missing").exists());
}

#[cfg(unix)]
#[test]
fn creates_completion_under_linked_config_directory() {
    use std::os::unix::fs::symlink;
    let home = tempfile::tempdir().unwrap();
    let target = home.path().join("dotfiles");
    fs::create_dir(&target).unwrap();
    symlink("dotfiles", home.path().join(".config")).unwrap();
    ok(run(
        home.path(),
        &["install", "--shell", "fish", "--dry-run"],
    ));
    assert!(!target.join("fish").exists());
    ok(run(home.path(), &["install", "--shell", "fish", "--yes"]));
    assert!(target.join("fish/completions/bs.fish").is_file());
    ok(run(home.path(), &["uninstall", "--shell", "fish", "--yes"]));
    assert!(!target.join("fish/completions/bs.fish").exists());
    assert_eq!(
        fs::read_link(home.path().join(".config")).unwrap(),
        Path::new("dotfiles")
    );
}
