//! Explicit, opt-in shell setup. Never reads or initializes a bitshelf store.
use crate::{
    cli::{Bs, Completion},
    usage_check,
};
use anyhow::{Context, Result, bail, ensure};
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

const START: &str = "# >>> bitshelf completion >>>\n";
const END: &str = "# <<< bitshelf completion <<<\n";

pub fn run(args: &Completion) -> Result<()> {
    let Some(action) = args.action.as_deref() else {
        usage_check(
            !args.dry_run && !args.yes,
            "--dry-run and --yes require install or uninstall",
        )?;
        let shell = args
            .shell
            .as_deref()
            .context("script generation requires --shell SHELL")?;
        print!(
            "{}",
            Bs::completion_script(
                usage::complete::Shell::from_name(shell).context("unsupported shell")?
            )
        );
        return Ok(());
    };
    let detected = env::var("SHELL").ok().and_then(|s| {
        Path::new(&s)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
    });
    let shell = args
        .shell
        .as_deref()
        .or(detected.as_deref())
        .context("cannot detect shell; pass --shell bash, zsh or fish")?;
    ensure!(
        matches!(shell, "bash" | "zsh" | "fish"),
        "automatic setup supports bash, zsh and fish; use bs completion --shell {shell} for manual setup"
    );
    let configured_path = target(shell)?;
    let path = resolve_target(&configured_path)?;
    if path != configured_path {
        println!(
            "Shell config: {} -> {}",
            configured_path.display(),
            path.display()
        );
    }
    let before = read_target(&path)?;
    let mut after = plan(shell, before.as_deref(), action == "install")?;
    // Keep a directly symlinked Fish file valid after uninstall. Removing its
    // contents unloads completion without breaking the user's dotfiles link.
    let linked_file =
        fs::symlink_metadata(&configured_path).is_ok_and(|metadata| metadata.is_symlink());
    if after.is_none() && before.is_some() && linked_file {
        after = Some(String::new());
    }
    if before == after {
        println!(
            "{}: {}",
            if action == "install" {
                "Already configured"
            } else {
                "No managed completion setup found"
            },
            path.display()
        );
        return Ok(());
    }
    println!("Will {action} completion setup in {}", path.display());
    if action == "install" {
        println!("{}", block(shell));
    } else {
        println!("Only bitshelf-managed completion setup will be removed.");
    }
    if args.dry_run {
        return Ok(());
    }
    if !args.yes {
        crate::interactive::require(false)
            .context("use --dry-run to preview or --yes to approve without a terminal")?;
        let confirmed = demand::Confirm::new("Apply completion setup changes?")
            .run()
            .map_err(|e| anyhow::anyhow!("prompt cancelled or failed: {e}"))?;
        ensure!(confirmed, "cancelled; no files changed");
    }
    // Do not clobber edits made while the preview/confirmation was displayed.
    ensure!(
        resolve_target(&configured_path)? == path && read_target(&path)? == before,
        "{} changed during setup; rerun",
        path.display()
    );
    match after {
        Some(content) => {
            if before.is_none() {
                fs::create_dir_all(path.parent().context("missing parent directory")?)?;
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?
                    .write_all(content.as_bytes())?;
            } else {
                // Preserve permissions and the inode of existing user configuration.
                fs::write(&path, content)?;
            }
        }
        None => fs::remove_file(&path)?,
    }
    println!(
        "Completion setup updated. {}",
        if action == "install" {
            "Start a new shell to activate it (the current shell is unchanged)."
        } else {
            "Start a new shell to unload it; manually installed or package-manager completions are unchanged."
        }
    );
    Ok(())
}

fn directory(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn target(shell: &str) -> Result<PathBuf> {
    let home = || directory("HOME").context("HOME is not set");
    let path = match shell {
        "bash" => home()?.join(".bashrc"),
        "zsh" => directory("ZDOTDIR")
            .map(Ok)
            .unwrap_or_else(home)?
            .join(".zshrc"),
        "fish" => directory("XDG_CONFIG_HOME")
            .map(Ok)
            .unwrap_or_else(|| home().map(|h| h.join(".config")))?
            .join("fish/completions/bs.fish"),
        _ => unreachable!(),
    };
    ensure!(
        path.is_absolute(),
        "shell configuration path must be absolute: {}",
        path.display()
    );
    Ok(path)
}

// Canonicalize existing links (including relative/chained links and linked
// parent directories). Resolve the nearest existing parent for new files too.
// A dangling link is an error, not permission to replace the link itself.
fn resolve_target(path: &Path) -> Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(_) => fs::canonicalize(path).with_context(|| {
            format!(
                "cannot resolve shell config {}; check for a broken or cyclic symlink",
                path.display()
            )
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = path.parent().context("shell config has no parent")?;
            let name = path.file_name().context("shell config has no filename")?;
            Ok(resolve_target(parent)?.join(name))
        }
        Err(error) => {
            Err(error).with_context(|| format!("resolving shell config {}", path.display()))
        }
    }
}

fn read_target(path: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            ensure!(
                meta.is_file() && !meta.is_symlink(),
                "shell config target {} is not a regular file or changed to a symlink; check the target and rerun",
                path.display()
            );
            fs::read_to_string(path)
                .with_context(|| format!("reading {}", path.display()))
                .map(Some)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

fn block(shell: &str) -> String {
    let body = match shell {
        "bash" => "eval \"$(bs completion --shell bash)\"\n".into(),
        "zsh" => "# Initialize completion only if it is not already available.\nif (( ! $+functions[compdef] )); then\n  autoload -Uz compinit && compinit\nfi\neval \"$(bs completion --shell zsh)\"\n".into(),
        "fish" => Bs::completion_script(usage::complete::Shell::from_name("fish").unwrap()),
        _ => unreachable!(),
    };
    format!("{START}{body}{END}")
}

fn plan(shell: &str, before: Option<&str>, install: bool) -> Result<Option<String>> {
    let old = before.unwrap_or("");
    let starts: Vec<_> = old.match_indices(START).map(|(i, _)| i).collect();
    let ends: Vec<_> = old.match_indices(END).map(|(i, _)| i).collect();
    let range = match (starts.as_slice(), ends.as_slice()) {
        ([], []) => None,
        ([start], [end]) if start < end && (*start == 0 || old.as_bytes()[start - 1] == b'\n') => {
            Some(*start..end + END.len())
        }
        _ => bail!(
            "malformed or duplicate bitshelf completion markers; repair configuration manually"
        ),
    };
    if shell == "fish" {
        if let Some(ref range) = range {
            ensure!(
                range.start == 0 && range.end == old.len(),
                "refusing to replace a fish completion file containing unmanaged content"
            );
        } else if before.is_some() && !old.is_empty() {
            if install {
                bail!(
                    "fish completion file already exists and is not bitshelf-managed; configure manually instead"
                );
            }
            return Ok(before.map(str::to_owned));
        }
        return Ok(install.then(|| block(shell)));
    }
    if let Some(range) = range {
        let mut result = old.to_owned();
        result.replace_range(range, &if install { block(shell) } else { String::new() });
        return Ok(Some(result));
    }
    if !install {
        return Ok(before.map(str::to_owned));
    }
    // Respect the exact manual activation lines recommended in previous docs.
    let eval = format!("eval \"$(bs completion --shell {shell})\"");
    let source = format!("source <(bs completion --shell {shell})");
    if old
        .lines()
        .any(|line| line.trim() == eval || line.trim() == source)
    {
        return Ok(before.map(str::to_owned));
    }
    let separator = if old.is_empty() || old.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    Ok(Some(format!("{old}{separator}{}", block(shell))))
}
