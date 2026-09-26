//! Identity validation shared by arguments and filesystem discovery.
use anyhow::{Context, Result, ensure};
use std::path::Path;

pub fn name(value: &str) -> Result<()> {
    ensure!(
        !value.trim().is_empty()
            && !value.starts_with('.')
            && !value.ends_with(".md")
            && !value
                .chars()
                .any(|c| c == '/' || c == '\\' || c.is_control()),
        "invalid name {value:?}: use a nonblank, nonhidden UTF-8 name without separators, controls or .md suffix"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composed_and_decomposed_components_are_byte_exact() {
        for shelf in ["caf\u{e9}", "cafe\u{301}"] {
            for name in ["r\u{e9}sum\u{e9}", "re\u{301}sume\u{301}"] {
                let id = format!("{shelf}/{name}");
                let parsed = BitId::parse(&id).unwrap();
                assert_eq!(parsed.shelf.as_bytes(), shelf.as_bytes());
                assert_eq!(parsed.name.as_bytes(), name.as_bytes());
                let file = format!("{name}.md");
                assert_eq!(
                    discovered(shelf, Path::new(&file)).unwrap().as_bytes(),
                    id.as_bytes()
                );
            }
        }
    }
}
