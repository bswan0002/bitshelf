//! Deliberately small template language: {{ variable }} only, always shell quoted.
use super::*;
use std::collections::BTreeSet;

pub(super) fn reserved(s: &str) -> bool {
    matches!(s, "args" | "store_path" | "config_path" | "bs_executable")
}
fn pieces(template: &str) -> Result<Vec<(&str, bool)>> {
    let mut rest = template;
    let mut out = Vec::new();
    while let Some(start) = rest.find("{{") {
        out.push((&rest[..start], false));
        let tail = &rest[start + 2..];
        let end = tail.find("}}").context("unclosed recipe placeholder")?;
        let key = tail[..end].trim();
        ensure!(
            key != "config",
            "recipe parameter config conflicts with bs --config; use config_path for context or choose another parameter name"
        );
        ensure!(
            !key.is_empty()
                && key.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                && !key.as_bytes()[0].is_ascii_digit(),
            "invalid recipe variable {key:?}; use {{{{ variable }}}}"
        );
        out.push((key, true));
        rest = &tail[end + 2..];
    }
    out.push((rest, false));
    Ok(out)
}
pub(super) fn recipe_variables(template: &str) -> Result<BTreeSet<String>> {
    Ok(pieces(template)?
        .into_iter()
        .filter(|(_, variable)| *variable)
        .map(|(s, _)| s.to_owned())
        .collect())
}
pub(super) fn render(
    d: &Definition,
    args: &[OsString],
    cfg: &crate::config::Config,
    path: &Path,
) -> Result<String> {
    let run = d.run.as_deref().unwrap();
    let vars = recipe_variables(run)?;
    let mut values = d.defaults.clone();
    let mut forwarded = Vec::new();
    let mut literal = false;
    for arg in args {
        let arg = arg.to_str().context("recipe arguments must be UTF-8")?;
        if !literal && arg == "--" {
            literal = true;
            continue;
        }
        if !literal
            && let Some((key, value)) = arg.strip_prefix("--").and_then(|s| s.split_once('='))
            && vars.contains(key)
            && !reserved(key)
        {
            values.insert(key.to_owned(), value.to_owned());
            continue;
        }
        forwarded.push(shell_words::quote(arg).into_owned());
    }
    values.insert(
        "store_path".into(),
        cfg.store
            .to_str()
            .context("store path must be UTF-8")?
            .into(),
    );
    values.insert(
        "config_path".into(),
        path.to_str().context("config path must be UTF-8")?.into(),
    );
    values.insert(
        "bs_executable".into(),
        std::env::current_exe()?
            .to_str()
            .context("executable path must be UTF-8")?
            .into(),
    );
    let mut result = String::new();
    for (text, variable) in pieces(run)? {
        if !variable {
            result.push_str(text);
        } else if text == "args" {
            result.push_str(&forwarded.join(" "));
        } else {
            let value = values
                .get(text)
                .with_context(|| format!("missing recipe parameter --{text}=VALUE"))?;
            result.push_str(&shell_words::quote(value));
        }
    }
    Ok(result)
}
