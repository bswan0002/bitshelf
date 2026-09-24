use anyhow::{Context, Result};
use std::{fs, io::Read, path::Path};

/// Body-only UTF-8 input; None means no replacement was requested.
pub fn body(file: Option<&Path>, stdin: bool) -> Result<Option<String>> {
    if stdin || file == Some(Path::new("-")) {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        Ok(Some(text))
    } else {
        file.map(|path| {
            fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))
        })
        .transpose()
    }
}
