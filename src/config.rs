use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    pub aliases: BTreeMap<String, crate::aliases::Alias>,
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
    /// Write settings. An existing file is edited in place so comments and
    /// layout survive; unchanged settings leave the file untouched.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let before = match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let text = match &before {
            None => toml::to_string_pretty(self)?,
            Some(bytes) => match self.edit(std::str::from_utf8(bytes)?, path)? {
                Some(text) => text,
                None => return Ok(()),
            },
        };
        crate::filesystem::publish(path, text.as_bytes(), before.as_deref(), None)?;
        Ok(())
    }
    /// None means the existing document already has exactly these settings.
    fn edit(&self, text: &str, path: &Path) -> Result<Option<String>> {
        let current: Self = toml::from_str(text)
            .with_context(|| format!("malformed shelf configuration {}", path.display()))?;
        if &current == self {
            return Ok(None);
        }
        if current.tag_rules != self.tag_rules {
            return Ok(Some(toml::to_string_pretty(self)?));
        }
        let mut doc: toml_edit::DocumentMut = text
            .parse()
            .with_context(|| format!("malformed shelf configuration {}", path.display()))?;
        fn set(doc: &mut toml_edit::DocumentMut, key: &str, value: Option<toml_edit::Value>) {
            match value {
                None => {
                    doc.remove(key);
                }
                Some(mut value) => {
                    // Replace only the value so the key's own decoration
                    // (including leading comments) and inline comments survive.
                    match doc.get_mut(key).and_then(|item| item.as_value_mut()) {
                        Some(old) => {
                            *value.decor_mut() = old.decor().clone();
                            *old = value;
                        }
                        None => {
                            doc.insert(key, toml_edit::Item::Value(value));
                        }
                    }
                }
            }
        }
        let unchanged_default = |present: bool, default: bool| !present && default;
        if !unchanged_default(doc.contains_key("discoverable"), self.discoverable) {
            set(&mut doc, "discoverable", Some(self.discoverable.into()));
        }
        set(
            &mut doc,
            "description",
            self.description.clone().map(Into::into),
        );
        if !unchanged_default(doc.contains_key("required"), self.required.is_empty()) {
            let required: toml_edit::Array = self.required.iter().cloned().collect();
            set(&mut doc, "required", Some(required.into()));
        }
        set(
            &mut doc,
            "retention",
            self.retention.clone().map(Into::into),
        );
        let text = doc.to_string();
        ensure!(
            &toml::from_str::<Self>(&text)? == self,
            "could not update {} in place",
            path.display()
        );
        Ok(Some(text))
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
    crate::identity::name(s)
}

/// About 100 years; longer periods are indistinguishable from permanent.
pub const MAX_RETENTION_DAYS: i64 = 36_500;
pub fn retention(s: &str) -> Result<chrono::Duration> {
    let digits = s
        .strip_suffix('d')
        .filter(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
        .context("retention must be positive whole days, e.g. 14d")?;
    let days: i64 = digits.parse().context("retention duration is too large")?;
    ensure!(days > 0, "retention must be positive");
    ensure!(
        days <= MAX_RETENTION_DAYS,
        "retention must be at most {MAX_RETENTION_DAYS}d"
    );
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
    // The XDG base directory specification ignores empty and relative values.
    let root = match env::var_os("XDG_CONFIG_HOME").map(PathBuf::from) {
        Some(v) if v.is_absolute() => v,
        _ => resolve(Path::new("~/.config"), &cwd)?,
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
        let before = if new { None } else { Some(fs::read(path)?) };
        crate::filesystem::publish(
            path,
            toml::to_string_pretty(self)?.as_bytes(),
            before.as_deref(),
            None,
        )?;
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
