use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShelfConfig {
    pub description: Option<String>,
    #[serde(default)]
    pub required: Vec<String>,
    pub retention: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub store: PathBuf,
    #[serde(default, deserialize_with = "deserialize_editor")]
    pub editor: Option<Vec<String>>,
    #[serde(default)]
    pub shelves: BTreeMap<String, ShelfConfig>,
}
fn deserialize_editor<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(
        untagged,
        expecting = "an editor command string or an array of strings"
    )]
    enum Editor {
        Command(String),
        Arguments(Vec<String>),
    }
    let args = match Editor::deserialize(deserializer)? {
        Editor::Command(command) => shell_words::split(&command).map_err(|err| {
            serde::de::Error::custom(format!("invalid quoted editor command: {err}"))
        })?,
        Editor::Arguments(args) => args,
    };
    Ok(Some(args))
}

pub fn name(s: &str) -> Result<()> {
    ensure!(
        !s.trim().is_empty()
            && !s.starts_with('.')
            && !s.contains(['/', '\\'])
            && !s.chars().any(char::is_control),
        "invalid name {s:?}: use a non-hidden single path component without separators or control characters"
    );
    Ok(())
}
pub fn retention(s: &str) -> Result<chrono::Duration> {
    let days: i64 = s
        .strip_suffix('d')
        .context("retention must be positive whole days, e.g. 14d")?
        .parse()
        .context("invalid retention duration")?;
    ensure!(days > 0, "retention must be positive");
    chrono::Duration::try_days(days).context("retention duration is too large")
}
pub fn resolve(p: &Path, base: &Path) -> Result<PathBuf> {
    let text = p.to_string_lossy();
    if text == "~" || text.starts_with("~/") {
        let home = env::var_os("HOME").context("HOME is unset")?;
        return Ok(PathBuf::from(home).join(text.strip_prefix("~/").unwrap_or("")));
    }
    Ok(if p.is_absolute() {
        p.to_owned()
    } else {
        base.join(p)
    })
}
pub fn config_path(p: Option<&Path>) -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    if let Some(p) = p {
        return resolve(p, &cwd);
    }
    let root = match env::var_os("XDG_CONFIG_HOME") {
        Some(v) => resolve(Path::new(&v), &cwd)?,
        None => resolve(Path::new("~/.config"), &cwd)?,
    };
    Ok(root.join("bitshelf/config.toml"))
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.store.as_os_str().is_empty(),
            "store must not be empty"
        );
        if let Some(editor) = &self.editor {
            ensure!(
                !editor.is_empty() && !editor[0].is_empty(),
                "editor must contain an executable"
            );
        }
        for (n, s) in &self.shelves {
            name(n)?;
            for r in &s.required {
                if !["title", "tags", "created", "updated", "expires"].contains(&r.as_str()) {
                    bail!("shelf {n}: unsupported required field {r}");
                }
            }
            if let Some(r) = &s.retention {
                retention(r).with_context(|| format!("shelf {n}"))?;
            }
        }
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).with_context(|| {
            format!(
                "cannot read config {}; run bs init --store PATH --editor 'code --wait'",
                path.display()
            )
        })?;
        let mut c: Self = toml::from_str(&text).context("malformed configuration")?;
        c.validate()?;
        c.store = resolve(&c.store, path.parent().unwrap())?;
        Ok(c)
    }
    pub fn save(&self, path: &Path, new: bool) -> Result<()> {
        self.validate()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let mut tmp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        use std::io::Write;
        tmp.write_all(toml::to_string_pretty(self)?.as_bytes())?;
        if new {
            tmp.persist_noclobber(path)
                .context("configuration already exists; it was not overwritten")?;
        } else {
            tmp.persist(path).context("cannot save configuration")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(editor: &str) -> Result<Config> {
        let config: Config = toml::from_str(&format!("store = '/tmp/bitshelf'\n{editor}"))?;
        config.validate()?;
        Ok(config)
    }
    #[test]
    fn accepts_editor_strings_and_arrays() {
        for value in [r#"editor = "code""#, r#"editor = ["code"]"#] {
            assert_eq!(parse(value).unwrap().editor, Some(vec!["code".into()]));
        }
        for value in [
            r#"editor = "code --wait 'a b' '$HOME' '$(touch NEVER)'""#,
            r#"editor = ["code", "--wait", "a b", "$HOME", "$(touch NEVER)"]"#,
        ] {
            assert_eq!(
                parse(value).unwrap().editor.unwrap(),
                vec!["code", "--wait", "a b", "$HOME", "$(touch NEVER)"]
            );
        }
    }
    #[test]
    fn missing_editor_still_uses_environment_and_saves_as_array() {
        assert!(parse("").unwrap().editor.is_none());
        let config = parse(r#"editor = "code --wait""#).unwrap();
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains(r#"editor = ["code", "--wait"]"#));
        assert_eq!(
            toml::from_str::<Config>(&serialized).unwrap().editor,
            config.editor
        );
    }
    #[test]
    fn invalid_editors_have_actionable_errors() {
        for value in [
            r#"editor = """#,
            r#"editor = "   ""#,
            "editor = []",
            r#"editor = [""]"#,
        ] {
            assert!(
                parse(value)
                    .unwrap_err()
                    .to_string()
                    .contains("editor must contain an executable")
            );
        }
        assert!(
            parse(r#"editor = "code 'unclosed""#)
                .unwrap_err()
                .to_string()
                .contains("invalid quoted editor command")
        );
        for value in [
            "editor = 42",
            "editor = [1]",
            "editor = { executable = 'code' }",
        ] {
            assert!(
                parse(value)
                    .unwrap_err()
                    .to_string()
                    .contains("an editor command string or an array of strings")
            );
        }
    }
}
