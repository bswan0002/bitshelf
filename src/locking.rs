//! Writer serialization independent of Markdown metadata/history.
use anyhow::{Context, Result, ensure};
use std::{fs, path::PathBuf};
pub struct Lock(PathBuf);
impl Lock {
    pub fn acquire(store: &crate::store::Store) -> Result<Self> {
        ensure!(store.config.store.is_dir(), "store does not exist");
        let dir = store.config.store.join(".bitshelf");
        store.safe(&dir)?;
        fs::create_dir_all(&dir)?;
        let path = dir.join("writer.lock");
        store.safe(&path)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .context("cannot acquire writer lock")?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
