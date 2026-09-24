use crate::store::Store;
use anyhow::{Context, Result};
use serde_json::{Value, json};
pub fn load(store: &Store, name: &str) -> Result<Value> {
    let path = store.shelf_path(name, true)?;
    let guidance_path = path.join("SHELF.md");
    store.safe(&guidance_path)?;
    let guidance = match std::fs::read_to_string(&guidance_path) {
        Ok(text) => json!({"path": guidance_path, "text": text}),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Value::Null,
        Err(e) => return Err(e).context("cannot read SHELF.md guidance"),
    };
    let cfg = store.settings(name);
    Ok(
        json!({"name": name, "path": path, "description": cfg.description, "required": cfg.required, "retention": cfg.retention, "guidance": guidance}),
    )
}
