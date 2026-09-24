use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShelfConfig {
    /// Participate in default discovery; explicit access is always available.
    #[serde(default = "default_discoverable")]
    pub discoverable: bool,
    pub description: Option<String>,
    #[serde(default)]
    pub required: Vec<String>,
    pub retention: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tag_rules: BTreeMap<String, TagRule>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TagRule {
    #[serde(default)]
    pub required: bool,
    pub allowed: Vec<String>,
}

fn valid_tag_component(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == ':' || c == ',')
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub aliases: BTreeMap<String, Vec<String>>,
    pub store: PathBuf,
    #[serde(default, deserialize_with = "deserialize_editor")]
    pub editor: Option<Vec<String>>,
}
fn default_discoverable() -> bool {
    true
}
impl Default for ShelfConfig {
    fn default() -> Self {
        Self {
            discoverable: true,
            description: None,
            required: vec![],
            retention: None,
            tag_rules: BTreeMap::new(),
        }
    }
}
impl ShelfConfig {
    pub fn validate(&self) -> Result<()> {
        for r in &self.required {
            ensure!(
                ["title", "tags", "created", "updated", "expires"].contains(&r.as_str()),
                "unsupported required field {r}"
            );
        }
        for (namespace, rule) in &self.tag_rules {
            ensure!(
                valid_tag_component(namespace),
                "invalid tag namespace {namespace:?}: expected a nonempty string without whitespace, colons, commas, or control characters"
            );
            ensure!(
                !rule.allowed.is_empty(),
                "tag_rules.{namespace}.allowed must not be empty"
            );
            let mut seen = std::collections::BTreeSet::new();
            for value in &rule.allowed {
                ensure!(
                    valid_tag_component(value),
                    "invalid allowed value {value:?} for tag namespace {namespace}: expected a nonempty string without whitespace, colons, commas, or control characters"
                );
                ensure!(
                    seen.insert(value),
                    "duplicate allowed value {value:?} for tag namespace {namespace}"
                );
            }
        }
        if let Some(r) = &self.retention {
            retention(r)?;
        }
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e).with_context(|| format!("cannot read {}", path.display())),
        };
        let cfg: Self = toml::from_str(&text)
            .with_context(|| format!("malformed shelf configuration {}", path.display()))?;
        cfg.validate()
            .with_context(|| format!("invalid shelf configuration {}", path.display()))?;
        Ok(cfg)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let mut tmp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        use std::io::Write;
        tmp.write_all(toml::to_string_pretty(self)?.as_bytes())?;
        tmp.persist(path)
            .context("cannot save shelf configuration")?;
        Ok(())
    }
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
        crate::aliases::validate(&self.aliases)?;
        if let Some(editor) = &self.editor {
            ensure!(
                !editor.is_empty() && !editor[0].is_empty(),
                "editor must contain an executable"
            );
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
    #[test]
    fn tag_rule_configuration_validation() {
        for text in [
            "[tag_rules.project]\nallowed = []",
            "[tag_rules.project]\nrequired = true",
            "[tag_rules.project]\nallowed = ['a', 'a']",
            "[tag_rules.project]\nallowed = ['']",
            "[tag_rules.project]\nallowed = ['a:b']",
            "[tag_rules.project]\nallowed = ['a,b']",
            "[tag_rules.project]\nallowed = ['a b']",
            "[tag_rules.'']\nallowed = ['a']",
            "[tag_rules.'a:b']\nallowed = ['a']",
            "[tag_rules.project]\nallowed = ['a']\nunknown = true",
        ] {
            assert!(
                toml::from_str::<ShelfConfig>(text)
                    .and_then(|cfg| { cfg.validate().map_err(serde::de::Error::custom) })
                    .is_err(),
                "{text}"
            );
        }
        let cfg: ShelfConfig =
            toml::from_str("[tag_rules.project]\nallowed = ['bitshelf']").unwrap();
        cfg.validate().unwrap();
        assert!(!cfg.tag_rules["project"].required);
        let serialized = toml::to_string(&cfg).unwrap();
        let roundtrip: ShelfConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(roundtrip.tag_rules["project"].allowed, ["bitshelf"]);
    }
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
