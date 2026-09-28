use crate::{
    bit::{self, Bit},
    config::{self, Config, ShelfConfig},
};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct DiscoveryError {
    pub path: String,
    pub error: String,
}
#[derive(Serialize)]
pub struct Collection<T> {
    pub results: Vec<T>,
    pub errors: Vec<DiscoveryError>,
    pub complete: bool,
}
impl<T> Collection<T> {
    fn new() -> Self {
        Self {
            results: vec![],
            errors: vec![],
            complete: true,
        }
    }
    fn error(&mut self, path: &Path, error: impl std::fmt::Display) {
        self.complete = false;
        self.errors.push(DiscoveryError {
            path: path.to_string_lossy().into_owned(),
            error: error.to_string(),
        });
    }
}
#[derive(Serialize)]
pub struct Shelf {
    pub discoverable: bool,
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub configured: bool,
    pub missing: bool,
    pub guidance_available: bool,
    pub required: Vec<String>,
    pub retention: Option<String>,
    pub on_expire: Option<crate::config::ExpiryAction>,
}
#[derive(Serialize)]
pub struct Validation {
    pub id: Option<String>,
    #[serde(serialize_with = "diagnostic_path")]
    pub path: PathBuf,
    pub errors: Vec<String>,
    pub valid: bool,
}
impl Validation {
    fn new(id: Option<String>, path: PathBuf, errors: Vec<String>) -> Self {
        Self {
            id,
            path,
            valid: errors.is_empty(),
            errors,
        }
    }
    fn failure(path: PathBuf, error: impl std::fmt::Display) -> Self {
        Self::new(None, path, vec![error.to_string()])
    }
}
pub struct Store {
    pub config: Config,
}
/// Root entries that resolve to existing non-directories (for example a
/// README symlink) are auxiliary files. Dangling links remain diagnostics:
/// they may be an unavailable linked shelf.
fn links_to_non_directory(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|m| !m.is_dir())
}
/// On case-insensitive or normalizing filesystems, a variant spelling can open
/// an entry stored under different bytes. Return the stored name in that case.
fn stored_name(dir: &Path, name: &str) -> Result<Option<std::ffi::OsString>> {
    use std::os::unix::fs::MetadataExt;
    let target = match fs::symlink_metadata(dir.join(name)) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(e).with_context(|| format!("inspecting {}", dir.join(name).display()));
        }
    };
    let mut same = None;
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        if entry.file_name() == name {
            return Ok(None);
        }
        if same.is_none()
            && entry
                .metadata()
                .is_ok_and(|m| m.ino() == target.ino() && m.dev() == target.dev())
        {
            same = Some(entry.file_name());
        }
    }
    Ok(Some(
        same.unwrap_or_else(|| "a differently spelled entry".into()),
    ))
}
fn entry_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspecting {}", path.display())),
    }
}
impl Store {
    // Refuse all shelf/bit symlinks, including internal ones. The explicitly selected store root may be a symlink.
    pub fn safe(&self, path: &Path) -> Result<()> {
        let rel = path
            .strip_prefix(&self.config.store)
            .context("path escapes store")?;
        let mut p = self.config.store.clone();
        for component in rel.components() {
            ensure!(
                matches!(component, std::path::Component::Normal(_)),
                "path escapes store"
            );
            p.push(component);
            match fs::symlink_metadata(&p) {
                Ok(m) => ensure!(
                    !m.file_type().is_symlink(),
                    "refusing symlink: {}",
                    p.display()
                ),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
    pub fn shelf_path(&self, name: &str, exists: bool) -> Result<PathBuf> {
        config::name(name)?;
        let p = self.config.store.join(name);
        self.safe(&p)?;
        if let Some(stored) = stored_name(&self.config.store, name)? {
            anyhow::bail!(
                "shelf {name} does not exactly match stored shelf {}; use the exact name",
                stored.to_string_lossy()
            );
        }
        if exists && !p.is_dir() {
            if name.starts_with('-') {
                anyhow::bail!("missing shelf {name}");
            }
            anyhow::bail!("missing shelf {name}; create it with bs shelf add -- {name}");
        }
        Ok(p)
    }
    pub fn bits_path(&self, shelf: &str) -> Result<PathBuf> {
        let path = self.shelf_path(shelf, true)?.join("bits");
        self.safe(&path)?;
        ensure!(
            path.is_dir(),
            "missing bits directory for shelf {shelf}; use bs shelf add {shelf}"
        );
        Ok(path)
    }
    pub fn settings(&self, shelf: &str) -> Result<ShelfConfig> {
        let path = self.shelf_path(shelf, false)?.join("bs.toml");
        self.safe(&path)?;
        ShelfConfig::load(&path)
    }
    pub fn shelves(&self) -> Result<Collection<Shelf>> {
        ensure!(
            self.config.store.is_dir(),
            "store does not exist: {}; run bs init",
            self.config.store.display()
        );
        let mut out = Collection::new();
        for entry in fs::read_dir(&self.config.store)? {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    out.error(&self.config.store, e);
                    continue;
                }
            };
            let path = entry.path();
            let filename = entry.file_name();
            if filename.as_encoded_bytes().starts_with(b".") {
                continue;
            }
            let result = (|| -> Result<Option<Shelf>> {
                let kind = entry.file_type()?;
                if kind.is_symlink() && links_to_non_directory(&path) {
                    return Ok(None);
                }
                self.safe(&path)?;
                if !kind.is_dir() {
                    return Ok(None);
                }
                if !entry_exists(&path.join("bits"))? && !entry_exists(&path.join("bs.toml"))? {
                    return Ok(None);
                }
                let name = filename
                    .to_str()
                    .context("unsupported non-UTF-8 shelf name")?;
                config::name(name)?;
                for field in ["bits", "bs.toml", "SHELF.md"] {
                    self.safe(&path.join(field))?;
                }
                let cfg = self.settings(name)?;
                let missing = !path.join("bits").is_dir();
                Ok(Some(Shelf {
                    name: name.into(),
                    path: path.clone(),
                    discoverable: cfg.discoverable,
                    configured: path.join("bs.toml").is_file(),
                    missing,
                    guidance_available: entry_exists(&path.join("SHELF.md"))?,
                    description: cfg.description,
                    required: cfg.required,
                    retention: cfg.retention,
                    on_expire: cfg.on_expire,
                }))
            })();
            match result {
                Ok(Some(s)) => out.results.push(s),
                Ok(None) => (),
                Err(e) => out.error(&path, format!("{e:#}")),
            }
        }
        out.results.sort_by(|a, b| a.name.cmp(&b.name));
        out.errors.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }
    pub fn bit_path(&self, id: &str) -> Result<PathBuf> {
        let parsed = crate::identity::BitId::parse(id)?;
        let bits = self.bits_path(parsed.shelf)?;
        let filename = format!("{}.md", parsed.name);
        let p = bits.join(&filename);
        self.safe(&p)?;
        if let Some(stored) = stored_name(&bits, &filename)? {
            let stored = stored.to_string_lossy();
            match stored.strip_suffix(".md") {
                Some(stem) => anyhow::bail!(
                    "{id} does not exactly match stored bit {}/{stem}; use the exact ID",
                    parsed.shelf
                ),
                None => anyhow::bail!(
                    "{id} resolves to stored file {stored}, which is not a supported bit name"
                ),
            }
        }
        Ok(p)
    }
    /// Resolve an ID that must name an existing regular bit file.
    pub fn existing_bit(&self, id: &str) -> Result<PathBuf> {
        let p = self.bit_path(id)?;
        match fs::symlink_metadata(&p) {
            Ok(m) if m.is_file() => Ok(p),
            Ok(_) => anyhow::bail!("bit {id} is not a regular file: {}", p.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => anyhow::bail!("missing bit {id}"),
            Err(e) => Err(e).with_context(|| format!("inspecting bit {id}")),
        }
    }
    pub fn discover(&self, shelf: Option<&str>, all: bool) -> Result<Collection<Bit>> {
        self.read_bits(shelf, !all)
    }
    pub fn bits(&self, shelf: Option<&str>) -> Result<Collection<Bit>> {
        self.read_bits(shelf, false)
    }
    fn read_bits(&self, shelf: Option<&str>, discovery: bool) -> Result<Collection<Bit>> {
        let mut out = Collection::new();
        let names = if let Some(s) = shelf {
            self.bits_path(s)?;
            self.settings(s)?;
            vec![s.to_owned()]
        } else {
            let shelves = self.shelves()?;
            out.errors = shelves.errors;
            out.complete = shelves.complete;
            shelves
                .results
                .into_iter()
                .filter(|s| !discovery || s.discoverable)
                .map(|s| s.name)
                .collect()
        };
        for name in names {
            let scope = (|| -> Result<()> {
                let cfg = self.settings(&name)?;
                for entry in fs::read_dir(self.bits_path(&name)?)? {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(e) => {
                            out.error(&self.config.store.join(&name), e);
                            continue;
                        }
                    };
                    let path = entry.path();
                    if entry.file_name().as_encoded_bytes().starts_with(b".")
                        || path.extension().is_none_or(|x| x != "md")
                    {
                        continue;
                    }
                    let result = (|| -> Result<Bit> {
                        self.safe(&path)?;
                        ensure!(entry.file_type()?.is_file(), "not a regular .md file");
                        let id = crate::identity::discovered(&name, &path)?;
                        let raw = fs::read_to_string(&path)?;
                        Ok(bit::inspect(id, path.clone(), &raw, &cfg))
                    })();
                    match result {
                        Ok(bit) => {
                            for error in &bit.errors {
                                out.error(&path, error);
                            }
                            out.results.push(bit);
                        }
                        Err(e) => out.error(&path, format!("{e:#}")),
                    }
                }
                Ok(())
            })();
            if let Err(error) = scope {
                out.error(&self.config.store.join(&name), format!("{error:#}"));
            }
        }
        out.results.sort_by(|a, b| a.id.cmp(&b.id));
        out.errors.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }
    /// Unlike discovery, validation accounts for disallowed links instead of
    /// silently excluding them. Inspect entries themselves, never their targets.
    pub fn validate(&self, shelf: Option<&str>) -> Result<Vec<Validation>> {
        let paths = if let Some(name) = shelf {
            config::name(name)?;
            vec![self.config.store.join(name)]
        } else {
            let mut paths = vec![];
            for entry in fs::read_dir(&self.config.store)? {
                let entry = entry?;
                if entry.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }
                let kind = entry.file_type()?;
                let path = entry.path();
                // Include root links that may be linked shelves (directories or
                // dangling targets); links to ordinary files are auxiliary.
                if (kind.is_symlink() && !links_to_non_directory(&path))
                    || (kind.is_dir()
                        && (entry_exists(&path.join("bits"))?
                            || entry_exists(&path.join("bs.toml"))?))
                {
                    paths.push(path);
                }
            }
            paths.sort();
            paths
        };
        let mut results = vec![];
        for path in paths {
            if let Err(error) = self.safe(&path) {
                results.push(Validation::failure(path, error));
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) if config::name(name).is_ok() => name,
                _ => {
                    results.push(Validation::failure(path, "unsupported shelf name"));
                    continue;
                }
            };
            let mut blocked = false;
            for field in ["bits", "bs.toml", "SHELF.md"] {
                let managed = path.join(field);
                if let Err(error) = self.safe(&managed) {
                    results.push(Validation::failure(managed, error));
                    blocked |= field != "SHELF.md";
                }
            }
            if blocked {
                continue;
            }
            let cfg = match self.settings(name) {
                Ok(cfg) => cfg,
                Err(error) => {
                    results.push(Validation::failure(
                        path.join("bs.toml"),
                        format!("{error:#}"),
                    ));
                    continue;
                }
            };
            let bits = match self.bits_path(name) {
                Ok(bits) => bits,
                Err(error) => {
                    results.push(Validation::failure(path, error));
                    continue;
                }
            };
            let mut entries = match fs::read_dir(&bits)
                .and_then(|entries| entries.collect::<std::io::Result<Vec<_>>>())
            {
                Ok(entries) => entries,
                Err(error) => {
                    results.push(Validation::failure(bits, error));
                    continue;
                }
            };
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path();
                if entry.file_name().to_string_lossy().starts_with('.')
                    || path.extension().is_none_or(|e| e != "md")
                {
                    continue;
                }
                let id = match crate::identity::discovered(name, &path) {
                    Ok(id) => id,
                    Err(error) => {
                        results.push(Validation::failure(path, error));
                        continue;
                    }
                };
                let errors = if let Err(error) = self.safe(&path) {
                    vec![error.to_string()]
                } else if entry.file_type()?.is_file() {
                    match fs::read_to_string(&path) {
                        Ok(raw) => bit::inspect(id.clone(), path.clone(), &raw, &cfg).errors,
                        Err(error) => vec![error.to_string()],
                    }
                } else {
                    vec!["not a regular .md file".into()]
                };
                results.push(Validation::new(Some(id), path, errors));
            }
        }
        Ok(results)
    }
    pub fn write_bit(&self, id: &str, raw: &str) -> Result<PathBuf> {
        let path = self.bit_path(id)?;
        crate::filesystem::publish(&path, raw.as_bytes(), None, None)?;
        Ok(path)
    }
}

fn diagnostic_path<S: serde::Serializer>(
    path: &Path,
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(&path.to_string_lossy())
}
