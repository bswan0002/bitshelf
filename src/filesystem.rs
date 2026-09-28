//! Local file commits. A post-publication durability error is not rollback.
use anyhow::{Context, Result, ensure};
use std::{fs, io::Write, path::Path};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stage {
    Temp,
    Write,
    Permissions,
    FileSync,
    Publish,
    DirectorySync,
    Remove,
}
#[cfg(test)]
thread_local! { static FAULT: std::cell::RefCell<Option<(std::path::PathBuf, Stage)>> = const { std::cell::RefCell::new(None) }; }
#[cfg(test)]
pub fn inject(path: &Path, stage: Stage) {
    FAULT.with(|f| *f.borrow_mut() = Some((path.into(), stage)));
}
fn check(path: &Path, stage: Stage) -> Result<()> {
    #[cfg(test)]
    FAULT.with(|f| {
        let mut f = f.borrow_mut();
        if f.as_ref().is_some_and(|(p, s)| p == path && *s == stage) {
            *f = None;
            anyhow::bail!("injected {stage:?} failure");
        }
        Ok(())
    })?;
    let _ = (path, stage);
    Ok(())
}
pub fn sync_parent(path: &Path) -> Result<()> {
    check(path, Stage::DirectorySync)?;
    fs::File::open(path.parent().context("missing parent")?)?.sync_all()?;
    Ok(())
}
fn regular(path: &Path) -> Result<fs::Metadata> {
    let meta = fs::symlink_metadata(path)?;
    ensure!(
        meta.is_file() && !meta.is_symlink(),
        "not a regular file: {}",
        path.display()
    );
    Ok(meta)
}
pub fn unchanged(path: &Path, before: &[u8]) -> Result<()> {
    regular(path)?;
    ensure!(
        fs::read(path)? == before,
        "{} changed during operation; rerun (no changes overwritten)",
        path.display()
    );
    Ok(())
}
/// None creates without clobbering; Some checks and replaces a regular file.
pub fn publish(
    path: &Path,
    bytes: &[u8],
    before: Option<&[u8]>,
    permissions: Option<fs::Permissions>,
) -> Result<()> {
    let existing = if let Some(before) = before {
        unchanged(path, before)?;
        Some(regular(path)?.permissions())
    } else {
        None
    };
    check(path, Stage::Temp)?;
    let mut tmp = tempfile::NamedTempFile::new_in(path.parent().context("missing parent")?)?;
    check(path, Stage::Write)?;
    tmp.write_all(bytes)?;
    check(path, Stage::Permissions)?;
    if let Some(permissions) = permissions.or(existing) {
        tmp.as_file().set_permissions(permissions)?;
    }
    check(path, Stage::FileSync)?;
    tmp.as_file().sync_all()?;
    check(path, Stage::Publish)?;
    if let Some(before) = before {
        unchanged(path, before)?;
        tmp.persist(path).context("replacement was not published")?;
    } else {
        tmp.persist_noclobber(path)
            .context("destination already exists or cannot be created; not overwritten")?;
    }
    sync_parent(path).with_context(|| {
        format!(
            "{} was saved, but parent-directory durability is uncertain; inspect before retrying",
            path.display()
        )
    })
}
#[derive(Debug)]
pub struct RemovalUncertain(pub std::path::PathBuf);
impl std::fmt::Display for RemovalUncertain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} was removed, but parent-directory durability is uncertain",
            self.0.display()
        )
    }
}
impl std::error::Error for RemovalUncertain {}

pub fn remove(path: &Path, before: &[u8]) -> Result<()> {
    unchanged(path, before)?;
    check(path, Stage::Remove)?;
    fs::remove_file(path)?;
    sync_parent(path).context(RemovalUncertain(path.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn faults_preserve_original_until_publication_and_report_afterwards() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("file");
        for stage in [
            Stage::Temp,
            Stage::Write,
            Stage::Permissions,
            Stage::FileSync,
            Stage::Publish,
        ] {
            fs::write(&path, b"before").unwrap();
            inject(&path, stage);
            assert!(publish(&path, b"after", Some(b"before"), None).is_err());
            assert_eq!(fs::read(&path).unwrap(), b"before");
            assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 1);
        }
        inject(&path, Stage::DirectorySync);
        assert!(
            publish(&path, b"after", Some(b"before"), None)
                .unwrap_err()
                .to_string()
                .contains("was saved")
        );
        assert_eq!(fs::read(&path).unwrap(), b"after");
        assert!(publish(&path, b"clobber", None, None).is_err());
        assert!(remove(&path, b"stale").is_err());
        inject(&path, Stage::Remove);
        assert!(remove(&path, b"after").is_err());
        assert!(path.exists());
        inject(&path, Stage::DirectorySync);
        assert!(
            remove(&path, b"after")
                .unwrap_err()
                .to_string()
                .contains("was removed")
        );
        assert!(!path.exists());
    }
}
