//! Global shortcuts, shell recipes, and explicitly invoked PATH extensions.
mod discovery;
mod dispatch;
mod recipes;
use anyhow::{Context, Result, ensure};
pub use dispatch::{dispatch, inspect, load_config};
use recipes::{recipe_variables, reserved};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Alias {
    Builtin(Vec<String>),
    Recipe(String),
    Detailed(Definition),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub argv: Option<Vec<String>>,
    pub exec: Option<Vec<String>>,
    pub run: Option<String>,
    pub description: Option<String>,
    pub usage: Option<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
}
impl Alias {
    fn definition(&self) -> Definition {
        match self {
            Self::Builtin(argv) => Definition {
                argv: Some(argv.clone()),
                ..Default::default()
            },
            Self::Recipe(run) => Definition {
                run: Some(run.clone()),
                ..Default::default()
            },
            Self::Detailed(d) => d.clone(),
        }
    }
}
pub const BUILTINS: &[&str] = &[
    "init",
    "shelf",
    "add",
    "edit",
    "move",
    "aliases",
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

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
pub fn validate(aliases: &BTreeMap<String, Alias>) -> Result<()> {
    for (name, alias) in aliases {
        ensure!(valid_name(name), "invalid alias name {name:?}");
        ensure!(
            !BUILTINS.contains(&name.as_str()),
            "alias {name} conflicts with a built-in command"
        );
        let d = alias.definition();
        ensure!(
            usize::from(d.argv.is_some())
                + usize::from(d.exec.is_some())
                + usize::from(d.run.is_some())
                == 1,
            "alias {name} requires exactly one of argv, exec, run"
        );
        if let Some(argv) = &d.argv {
            ensure!(
                argv.first()
                    .is_some_and(|a| BUILTINS.contains(&a.as_str()) && a != "help"),
                "alias {name} must begin with a built-in command (aliases cannot chain)"
            );
            for arg in argv {
                expand(arg, "shelf/name").with_context(|| format!("invalid alias {name}"))?;
            }
        }
        if let Some(exec) = &d.exec {
            ensure!(
                exec.first().is_some_and(|s| !s.trim().is_empty()),
                "helper alias {name} must contain an executable"
            );
        }
        if let Some(run) = &d.run {
            ensure!(!run.trim().is_empty(), "recipe {name} must not be empty");
            let vars = recipe_variables(run)?;
            for key in d.defaults.keys() {
                ensure!(
                    vars.contains(key) && !reserved(key),
                    "unused or reserved recipe default {key}"
                );
            }
        } else {
            ensure!(
                d.defaults.is_empty(),
                "defaults are only supported for recipes"
            );
        }
    }
    Ok(())
}
/// Bind the trusted template before substitution so an ID cannot become an option.
/// Use the CLI's own grammar, not a second list of flags and their arities.
fn expand_template(template: &[String], id: &str, forwarded: &[OsString]) -> Result<Vec<OsString>> {
    let argv: Vec<_> = template.iter().map(|s| s.as_ref()).collect();
    let mut options = Vec::new();
    let mut operands = Vec::new();
    let mut parser = usage::Parser::new(crate::cli::Bs::command(), &argv);
    while let Some(event) = parser.next_event() {
        let event =
            event.map_err(|e| crate::UsageError(format!("invalid alias arguments: {e:?}")))?;
        match event {
            usage::Event::Command(command) => options.push(command.name.into()),
            usage::Event::Flag {
                flag,
                value,
                negated,
            } => {
                let name = if negated {
                    format!(
                        "--{}",
                        flag.negate.context("alias flag has no negated form")?
                    )
                } else if let Some(long) = flag.longs.first() {
                    format!("--{long}")
                } else {
                    format!(
                        "-{}",
                        char::from(*flag.shorts.first().context("alias flag has no name")?)
                    )
                };
                options.push(match value {
                    Some(value) => {
                        format!("{name}={}", expand(std::str::from_utf8(value)?, id)?).into()
                    }
                    None => name.into(),
                });
            }
            usage::Event::Arg { value, .. } => {
                operands.push(expand(std::str::from_utf8(value)?, id)?.into());
            }
            _ => anyhow::bail!("unsupported alias template binding"),
        }
    }
    options.extend_from_slice(forwarded);
    options.push("--".into());
    options.extend(operands);
    Ok(options)
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
