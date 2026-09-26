//! Explicit global argv aliases: built-in expansions or opted-in helper executables.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Alias {
    Builtin(Vec<String>),
    External(ExternalAlias),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAlias {
    pub exec: Vec<String>,
}

const BUILTINS: &[&str] = &[
    "init",
    "shelf",
    "add",
    "edit",
    "move",
    "aliases",
    "sync",
    "list",
    "search",
    "show",
    "open",
    "context",
    "validate",
    "prune",
    "completion",
    "help",
];

fn expand(template: &str, id: &str) -> Result<String> {
    let (shelf, name) = id.split_once('/').unwrap_or(("", ""));
    let mut result = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        let tail = &rest[start + 1..];
        let end = tail.find('}').context("unclosed alias placeholder")?;
        result.push_str(match &tail[..end] {
            "id" => id,
            "shelf" => shelf,
            "name" => name,
            other => anyhow::bail!(
                "unknown alias placeholder {{{other}}}; use {{id}}, {{shelf}}, {{name}}"
            ),
        });
        rest = &tail[end + 1..];
    }
    result.push_str(rest);
    Ok(result)
}

pub fn validate(aliases: &BTreeMap<String, Alias>) -> Result<()> {
    for (name, alias) in aliases {
        ensure!(
            !name.is_empty()
                && name
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
                && !name.starts_with('-'),
            "invalid alias name {name:?}"
        );
        ensure!(
            !BUILTINS.contains(&name.as_str()),
            "alias {name} conflicts with a built-in command"
        );
        let args = match alias {
            Alias::Builtin(args) => args,
            Alias::External(helper) => {
                ensure!(
                    helper.exec.first().is_some_and(|s| !s.trim().is_empty()),
                    "helper alias {name} must contain an executable"
                );
                continue;
            }
        };
        ensure!(
            args.first()
                .is_some_and(|a| BUILTINS.contains(&a.as_str()) && a != "help"),
            "alias {name} must begin with a built-in command (aliases cannot chain)"
        );
        for arg in args {
            expand(arg, "shelf/name").with_context(|| format!("invalid alias {name}"))?;
        }
    }
    Ok(())
}

/// Intercept only a top-level alias, leaving built-in parsing and completion intact.
pub fn dispatch() -> Result<()> {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let mut config = None;
    let mut global_json = false;
    let mut index = 0;
    while index < args.len() {
        let word = args[index].to_string_lossy();
        if word == "--config" {
            index += 1;
            config = args.get(index).map(PathBuf::from);
            if config.is_none() {
                return Ok(());
            }
        } else if let Some(path) = word.strip_prefix("--config=") {
            config = Some(PathBuf::from(path));
        } else if word == "--json" {
            global_json = true;
        } else {
            break;
        }
        index += 1;
    }
    let Some(name) = args.get(index).and_then(|s| s.to_str()) else {
        return Ok(());
    };
    if name.starts_with(['-', '_']) || BUILTINS.contains(&name) {
        return Ok(());
    }
    // Global options may follow the alias, just as they may follow built-ins.
    let mut forwarded = Vec::new();
    let mut positional = Vec::new();
    let mut i = index + 1;
    let mut literal = false;
    while i < args.len() {
        let word = args[i].to_string_lossy();
        if !literal && word == "--" {
            literal = true;
        } else if !literal && word == "--config" {
            i += 1;
            let path = args.get(i).context("--config requires a path")?;
            config = Some(PathBuf::from(path));
            forwarded.extend([OsString::from("--config"), path.clone()]);
        } else if !literal && word.starts_with("--config=") {
            config = Some(PathBuf::from(word.strip_prefix("--config=").unwrap()));
            forwarded.push(args[i].clone());
        } else if !literal && word.starts_with('-') {
            forwarded.push(args[i].clone());
        } else {
            positional.push(args[i].clone());
        }
        i += 1;
    }
    let path = crate::config::config_path(config.as_deref())?;
    let cfg = crate::config::Config::load(&path)?;
    let Some(alias) = cfg.aliases.get(name) else {
        return Ok(());
    };
    let template = match alias {
        Alias::Builtin(template) => template,
        Alias::External(helper) => {
            // Consume bs configuration options, but preserve all helper arguments
            // (including --help and literal arguments after --) without expansion.
            let mut helper_args: Vec<OsString> = if global_json {
                vec!["--json".into()]
            } else {
                vec![]
            };
            let mut i = index + 1;
            let mut literal = false;
            while i < args.len() {
                let word = args[i].to_string_lossy();
                if !literal && word == "--config" {
                    i += 2;
                    continue;
                }
                if !literal && word.starts_with("--config=") {
                    i += 1;
                    continue;
                }
                if word == "--" {
                    literal = true;
                }
                helper_args.push(args[i].clone());
                i += 1;
            }
            let status = std::process::Command::new(&helper.exec[0])
                .args(&helper.exec[1..])
                .args(helper_args)
                .env("BS_CONFIG", &path)
                .env("BS_EXECUTABLE", std::env::current_exe()?)
                .status()
                .with_context(|| format!("cannot execute helper alias {name}"))?;
            std::process::exit(status.code().unwrap_or(1));
        }
    };
    let templated = template.iter().any(|s| s.contains('{'));
    if forwarded.iter().any(|s| s == "--help" || s == "-h") {
        let usage = if templated {
            "ID [--dry-run] [--json]"
        } else {
            "[ARGS...]"
        };
        crate::output::write(format!("Usage: bs {name} {usage}\n\nExpands to argv: {}\nPlaceholders: {{id}}, {{shelf}}, {{name}}. No shell execution.\n", serde_json::to_string(template)?).as_bytes())?;
        std::process::exit(0);
    }
    let id = if templated {
        crate::usage_check(
            positional.len() == 1,
            &format!("bs {name} requires exactly one shelf/bit-name ID"),
        )?;
        for flag in &forwarded {
            // Configuration flags/values were already parsed above.
            let flag = flag.to_string_lossy();
            if flag.starts_with('-') {
                crate::usage_check(
                    ["--json", "--dry-run", "--config"].contains(&flag.as_ref())
                        || flag.starts_with("--config="),
                    "ID aliases accept only --json, --dry-run and --config; configure other arguments in the alias",
                )?;
            }
        }
        let id = positional[0].to_str().context("alias ID must be UTF-8")?;
        let parts: Vec<_> = id.split('/').collect();
        ensure!(parts.len() == 2, "expected shelf/bit-name identifier");
        for part in parts {
            crate::config::name(part)?;
        }
        id
    } else {
        forwarded = args[index + 1..].to_vec();
        ""
    };
    let expanded = template
        .iter()
        .map(|s| expand(s, id))
        .collect::<Result<Vec<_>>>()?;
    let status = std::process::Command::new(std::env::current_exe()?)
        .args(&args[..index])
        .args(expanded)
        .args(forwarded)
        .status()
        .context("cannot execute alias")?;
    std::process::exit(status.code().unwrap_or(1));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expansion_is_single_pass() {
        assert_eq!(
            expand("archive/{shelf}.{name}", "notes/{id}").unwrap(),
            "archive/notes.{id}"
        );
        assert!(expand("{unknown}", "notes/x").is_err());
        assert!(expand("{id", "notes/x").is_err());
    }
}
