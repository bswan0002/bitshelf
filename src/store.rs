use crate::{
    bit::{self, Bit},
    config::{self, Config, ShelfConfig},
};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct Shelf {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub configured: bool,
    pub missing: bool,
    pub guidance_available: bool,
    pub required: Vec<String>,
    pub retention: Option<String>,
}
pub struct Store {
    pub config: Config,
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
        if exists {
            ensure!(
                p.is_dir(),
                "missing shelf {name}; create it with bs shelf add {name}"
            );
        }
        Ok(p)
    }
    pub fn settings(&self, shelf: &str) -> ShelfConfig {
        self.config.shelves.get(shelf).cloned().unwrap_or_default()
    }
    pub fn shelves(&self) -> Result<Vec<Shelf>> {
        ensure!(
            self.config.store.is_dir(),
            "store does not exist: {}; run bs init",
            self.config.store.display()
        );
        let mut names: BTreeSet<String> = self.config.shelves.keys().cloned().collect();
        for e in fs::read_dir(&self.config.store)? {
            let e = e?;
            let n = e.file_name().to_string_lossy().into_owned();
            if !n.starts_with('.') && e.file_type()?.is_dir() {
                names.insert(n);
            }
        }
        names
            .into_iter()
            .map(|name| {
                let path = self.config.store.join(&name);
                self.safe(&path)?;
                let cfg = self.settings(&name);
                Ok(Shelf {
                    configured: self.config.shelves.contains_key(&name),
                    missing: !path.is_dir(),
                    guidance_available: path.join("SHELF.md").try_exists()?,
                    name,
                    path,
                    description: cfg.description,
                    required: cfg.required,
                    retention: cfg.retention,
                })
            })
            .collect()
    }
    pub fn bit_path(&self, id: &str) -> Result<PathBuf> {
        let parts: Vec<_> = id.split('/').collect();
        ensure!(
            parts.len() == 2,
            "expected shelf/slug identifier (without .md)"
        );
        config::name(parts[0])?;
        config::name(parts[1])?;
        ensure!(
            parts[1] != "SHELF",
            "SHELF.md is guidance, not a bit; use bs context"
        );
        let p = self
            .shelf_path(parts[0], true)?
            .join(format!("{}.md", parts[1]));
        self.safe(&p)?;
        Ok(p)
    }
    pub fn bits(&self, shelf: Option<&str>) -> Result<Vec<Bit>> {
        let shelves = if let Some(s) = shelf {
            self.shelf_path(s, true)?;
            vec![s.to_owned()]
        } else {
            self.shelves()?
                .into_iter()
                .filter_map(|s| {
                    if s.missing {
                        eprintln!(
                            "warning: missing shelf {}; use bs shelf add {}",
                            s.name, s.name
                        );
                        None
                    } else {
                        Some(s.name)
                    }
                })
                .collect()
        };
        let mut bits = vec![];
        for s in shelves {
            for e in fs::read_dir(self.shelf_path(&s, true)?)? {
                let e = e?;
                let p = e.path();
                if p.extension().is_none_or(|x| x != "md")
                    || e.file_name() == "SHELF.md"
                    || e.file_name().to_string_lossy().starts_with('.')
                {
                    continue;
                }
                if let Err(err) = self.safe(&p) {
                    eprintln!("warning: {err}");
                    continue;
                }
                if !e.file_type()?.is_file() {
                    continue;
                }
                let id = format!("{s}/{}", p.file_stem().unwrap().to_string_lossy());
                match fs::read_to_string(&p) {
                    Ok(raw) => {
                        let bit = bit::inspect(id, p, &raw, &self.settings(&s));
                        for err in &bit.errors {
                            eprintln!("warning: {}: {err}", bit.id);
                        }
                        bits.push(bit);
                    }
                    Err(err) => {
                        eprintln!("warning: {}: {err}", p.display());
                        bits.push(Bit {
                            id,
                            path: p,
                            title: String::new(),
                            tags: vec![],
                            metadata: serde_json::Value::Null,
                            errors: vec![err.to_string()],
                            body: String::new(),
                            expires: None,
                        });
                    }
                }
            }
        }
        bits.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(bits)
    }
    pub fn write_bit(&self, id: &str, raw: &str) -> Result<PathBuf> {
        let path = self.bit_path(id)?;
        let mut tmp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        tmp.write_all(raw.as_bytes())?;
        tmp.persist_noclobber(&path).with_context(|| {
            format!("cannot create {id}; if it exists, choose --slug ALTERNATIVE")
        })?;
        Ok(path)
    }
}
