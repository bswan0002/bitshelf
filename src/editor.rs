use crate::config::Config;
use anyhow::{Context, Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    Open,
    Draft,
}

/// Keep environment selection in one place for both setup and editor launching.
pub fn environment_editor() -> Option<String> {
    choose_environment(std::env::var("VISUAL").ok(), std::env::var("EDITOR").ok())
}
fn choose_environment(visual: Option<String>, editor: Option<String>) -> Option<String> {
    visual
        .filter(|s| !s.trim().is_empty())
        .or_else(|| editor.filter(|s| !s.trim().is_empty()))
}

/// Recognize executables, not arbitrary substrings inside shell/wrapper commands.
/// Returns false when the caller should explain the blocking requirement.
fn prepare(editor: &mut Vec<String>, purpose: Purpose) -> bool {
    if purpose == Purpose::Open {
        return true;
    }
    let executable = Path::new(&editor[0])
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    match executable {
        "code" | "code-insiders" | "codium" | "cursor" | "subl" => {
            if !editor.iter().skip(1).any(|a| a == "--wait" || a == "-w") {
                // Place before any `--` argument separator.
                editor.insert(1, "--wait".into());
            }
            true
        }
        "vi" | "vim" | "nvim" | "nano" | "pico" | "hx" | "helix" | "micro" => true,
        _ => false,
    }
}

pub fn launch(config: &Config, paths: &[PathBuf], json: bool, purpose: Purpose) -> Result<()> {
    let mut editor = if let Some(e) = &config.editor {
        e.clone()
    } else {
        let value = environment_editor()
            .context("no editor configured; set editor in config.toml, VISUAL, or EDITOR")?;
        shell_words::split(&value).context("invalid quoted editor arguments")?
    };
    ensure!(
        !editor.is_empty() && !editor[0].is_empty(),
        "editor command is empty"
    );
    if !prepare(&mut editor, purpose) {
        eprintln!(
            "Note: unrecognized editor command {:?}. For draft editing it must stay running until you finish; configure its wait flag if needed. Wrapper commands are not modified.",
            editor[0]
        );
    }
    let mut command = Command::new(&editor[0]);
    command.args(&editor[1..]).args(paths);
    if json {
        command.stdout(std::process::Stdio::from(std::io::stderr()));
    }
    let status = command
        .status()
        .with_context(|| format!("cannot launch editor {}", editor[0]))?;
    ensure!(status.success(), "editor exited with {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn words(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn known_gui_editors_wait_only_for_drafts() {
        for executable in [
            "code",
            "/Applications/bin/code",
            "code-insiders",
            "codium",
            "cursor",
            "subl",
        ] {
            let original = words(&[executable, "--reuse-window", "--"]);
            let mut command = original.clone();
            assert!(prepare(&mut command, Purpose::Open));
            assert_eq!(command, original);
            assert!(prepare(&mut command, Purpose::Draft));
            assert_eq!(
                command,
                words(&[executable, "--wait", "--reuse-window", "--"])
            );
        }
    }
    #[test]
    fn existing_wait_flags_are_not_duplicated() {
        for flag in ["-w", "--wait"] {
            let original = words(&["subl", flag]);
            let mut command = original.clone();
            assert!(prepare(&mut command, Purpose::Draft));
            assert_eq!(command, original);
        }
    }
    #[test]
    fn terminal_editors_and_unknown_wrappers_are_unchanged() {
        for (args, known) in [
            (&["vim", "-f"][..], true),
            (&["nvim"][..], true),
            (&["sh", "-c", "code"][..], false),
            (&["my-code-wrapper"][..], false),
        ] {
            let original = words(args);
            let mut command = original.clone();
            assert_eq!(prepare(&mut command, Purpose::Draft), known);
            assert_eq!(command, original);
        }
    }
    #[test]
    fn environment_precedence_and_empty_values() {
        assert_eq!(
            choose_environment(Some("vim".into()), Some("nano".into())),
            Some("vim".into())
        );
        assert_eq!(
            choose_environment(Some("  ".into()), Some("nano".into())),
            Some("nano".into())
        );
        assert_eq!(choose_environment(None, Some("".into())), None);
    }
}
