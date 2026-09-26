//! Identity validation shared by arguments and filesystem discovery.
use anyhow::{Context, Result, ensure};
use std::path::Path;

pub fn name(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && !value.starts_with('.')
            && !value.ends_with(".md")
            && !value
                .chars()
                .any(|c| c == '/' || c == '\\' || c.is_control()),
        "invalid name {value:?}: use a nonhidden UTF-8 name without separators, controls or .md suffix"
    );
    Ok(())
}

pub struct BitId<'a> {
    pub shelf: &'a str,
    pub name: &'a str,
}
impl<'a> BitId<'a> {
    pub fn parse(value: &'a str) -> Result<Self> {
        let (shelf, bit) = value
            .split_once('/')
            .context("expected shelf/bit-name identifier (without .md)")?;
        name(shelf)?;
        name(bit)?;
        Ok(Self { shelf, name: bit })
    }
}
pub fn discovered(shelf: &str, path: &Path) -> Result<String> {
    name(shelf)?;
    ensure!(
        path.extension().is_some_and(|e| e == "md"),
        "expected .md file"
    );
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("unsupported non-UTF-8 filename")?;
    name(stem)?;
    Ok(format!("{shelf}/{stem}"))
}
