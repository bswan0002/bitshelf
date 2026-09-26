//! OS advisory locks auto-release on process death; the persistent file is not
//! a sentinel and must never be unlinked while competing processes may use it.
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    time::{Duration, Instant},
};
pub struct Lock {
    _file: fs::File,
}
impl Lock {
    pub fn acquire(store: &crate::store::Store) -> Result<Self> {
        Self::acquire_for(store, Duration::from_secs(5))
    }
    fn acquire_for(store: &crate::store::Store, timeout: Duration) -> Result<Self> {
        ensure!(store.config.store.is_dir(), "store does not exist");
        let dir = store.config.store.join(".bitshelf");
        store.safe(&dir)?;
        fs::create_dir_all(&dir)?;
        let path = dir.join("writer.lock");
        store.safe(&path)?;
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("cannot open writer lock {}", path.display()))?;
        let start = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { _file: file }),
                Err(fs::TryLockError::WouldBlock) if start.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(25))
                }
                Err(fs::TryLockError::WouldBlock) => anyhow::bail!(
                    "writer lock timed out after {}ms at {}; another writer is active; retry after it exits (do not delete the lock file)",
                    timeout.as_millis(),
                    path.display()
                ),
                Err(fs::TryLockError::Error(e)) => {
                    return Err(e).context("OS writer locking is unavailable");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timeout_release_and_persistent_lock_file() {
        let tmp = tempfile::tempdir().unwrap();
        let store = crate::store::Store {
            config: crate::config::Config {
                store: tmp.path().into(),
                editor: None,
                aliases: Default::default(),
            },
        };
        let lock = Lock::acquire(&store).unwrap();
        assert!(Lock::acquire_for(&store, Duration::ZERO).is_err());
        drop(lock);
        assert!(tmp.path().join(".bitshelf/writer.lock").exists());
        Lock::acquire_for(&store, Duration::ZERO).unwrap();
    }
}
