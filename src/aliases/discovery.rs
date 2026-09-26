use super::*;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    description: Option<String>,
    usage: Option<String>,
    #[serde(default)]
    examples: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct Entry {
    pub name: String,
    pub kind: &'static str,
    pub origin: PathBuf,
    pub description: Option<String>,
    pub usage: Option<String>,
    pub examples: Vec<String>,
    pub argv: Option<Vec<String>>,
    pub exec: Option<Vec<String>>,
    pub run: Option<String>,
    pub parameters: Vec<String>,
    pub required_parameters: Vec<String>,
    pub defaults: BTreeMap<String, String>,
    pub metadata_error: Option<String>,
}

pub(super) fn executables() -> BTreeMap<String, PathBuf> {
    let mut found = BTreeMap::new();
    let Some(path) = std::env::var_os("PATH") else {
        return found;
    };
    let cwd = std::env::current_dir().unwrap_or_default();
    for dir in std::env::split_paths(&path) {
        let dir = if dir.is_absolute() {
            dir
        } else {
            cwd.join(dir)
        };
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let Some(name) = filename.to_str().and_then(|s| s.strip_prefix("bs-")) else {
                continue;
            };
            if !valid_name(name) || BUILTINS.contains(&name) {
                continue;
            }
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if !path
                    .metadata()
                    .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
                {
                    continue;
                }
            }
            #[cfg(not(unix))]
            {
                continue;
            }
            found.entry(name.to_owned()).or_insert(path);
        }
    }
    found
}

pub(super) fn catalog(cfg: &crate::config::Config, path: &Path) -> Result<BTreeMap<String, Entry>> {
    let mut entries = BTreeMap::new();
    for (name, executable) in executables() {
        if cfg.aliases.contains_key(&name) {
            continue;
        }
        let sidecar = executable.with_file_name(format!("bs-{name}.toml"));
        let metadata = match std::fs::read_to_string(&sidecar) {
            Ok(s) => toml::from_str::<Metadata>(&s).map_err(|e| e.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Metadata::default()),
            Err(e) => Err(e.to_string()),
        };
        let (m, error) = match metadata {
            Ok(m) => (m, None),
            Err(e) => (
                Metadata::default(),
                Some(format!("{}: {e}", sidecar.display())),
            ),
        };
        entries.insert(
            name.clone(),
            Entry {
                name,
                kind: "executable",
                origin: executable.clone(),
                description: m.description,
                usage: m.usage,
                examples: m.examples,
                argv: None,
                exec: Some(vec![executable.to_string_lossy().into_owned()]),
                run: None,
                parameters: vec![],
                required_parameters: vec![],
                defaults: BTreeMap::new(),
                metadata_error: error,
            },
        );
    }
    for (name, alias) in &cfg.aliases {
        let d = alias.definition();
        let parameters: Vec<String> = d
            .run
            .as_deref()
            .map(recipe_variables)
            .transpose()?
            .unwrap_or_default()
            .into_iter()
            .filter(|v| !reserved(v))
            .collect();
        let required_parameters = parameters
            .iter()
            .filter(|p| !d.defaults.contains_key(*p))
            .cloned()
            .collect();
        let usage = d.usage.clone().or_else(|| {
            d.argv.as_ref().map(|argv| {
                if argv.iter().any(|s| s.contains('{')) {
                    format!("{name} ID [--json] [--dry-run]")
                } else {
                    format!("{name} [ARGS...]")
                }
            })
        });
        entries.insert(
            name.clone(),
            Entry {
                name: name.clone(),
                kind: if d.run.is_some() {
                    "recipe"
                } else if d.exec.is_some() {
                    "helper"
                } else {
                    "argv"
                },
                origin: path.to_owned(),
                description: d.description,
                usage,
                examples: d.examples,
                argv: d.argv,
                exec: d.exec,
                run: d.run,
                parameters,
                required_parameters,
                defaults: d.defaults,
                metadata_error: None,
            },
        );
    }
    Ok(entries)
}
