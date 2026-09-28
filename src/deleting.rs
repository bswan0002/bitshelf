//! Explicit deletion and prune share the same lock-held guarded removal.
use crate::{cli::Delete, store::Store};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn remove_locked(store: &Store, path: &Path, before: &[u8], dry_run: bool) -> Result<()> {
    store.safe(path)?;
    if dry_run {
        crate::filesystem::unchanged(path, before)
    } else {
        crate::filesystem::remove(path, before)
    }
}

pub fn run(store: &Store, args: Delete) -> Result<(Vec<Value>, bool)> {
    ensure!(!args.id.is_empty(), "delete requires at least one ID");
    let _lock = if args.dry_run {
        None
    } else {
        Some(crate::locking::Lock::acquire(store)?)
    };
    let mut rows = vec![];
    let mut failed = false;
    let mut seen = std::collections::BTreeSet::new();
    for id in args.id {
        if !seen.insert(id.clone()) {
            continue;
        }
        let mut path = None;
        let result = (|| -> Result<()> {
            let source = store.existing_bit(&id)?;
            path = Some(source.clone());
            let before = fs::read(&source)?;
            remove_locked(store, &source, &before, args.dry_run)
        })();
        match result {
            Ok(()) => rows.push(json!({"id":id,"path":path,"status":if args.dry_run {"would_remove"} else {"removed"}})),
            Err(error) => {
                failed = true;
                let uncertain = error.downcast_ref::<crate::filesystem::RemovalUncertain>().is_some();
                rows.push(json!({"id":id,"path":path,"status":if uncertain {"removed_with_error"} else {"skipped"},"error":format!("{error:#}")}));
                if uncertain {
                    break;
                }
            }
        }
    }
    Ok((rows, failed))
}

// None is the empty bits directory; Some(bytes) is settings or guidance.
type ShelfSnapshot = std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>>;
fn shelf_snapshot(store: &Store, path: &Path) -> Result<ShelfSnapshot> {
    store.safe(path)?;
    let mut entries = ShelfSnapshot::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        store.safe(&path)?;
        let kind = entry.file_type()?;
        let bytes = if entry.file_name() == "bits" && kind.is_dir() {
            ensure!(
                fs::read_dir(&path)?.next().is_none(),
                "shelf is not empty: {}; delete or move its bits explicitly first",
                path.display()
            );
            None
        } else {
            ensure!(
                (entry.file_name() == "bs.toml" || entry.file_name() == "SHELF.md")
                    && kind.is_file(),
                "refusing shelf deletion: unexpected entry {} (only an empty bits directory, bs.toml, and SHELF.md are allowed)",
                path.display()
            );
            Some(fs::read(&path)?)
        };
        entries.insert(path, bytes);
    }
    Ok(entries)
}

pub fn shelf(store: &Store, args: crate::cli::ShelfDelete) -> Result<(Value, bool)> {
    shelf_with(store, args, |_| {})
}
fn shelf_with(
    store: &Store,
    args: crate::cli::ShelfDelete,
    mut before_remove: impl FnMut(&Path),
) -> Result<(Value, bool)> {
    let _lock = if args.dry_run {
        None
    } else {
        Some(crate::locking::Lock::acquire(store)?)
    };
    let mut removed = vec![];
    let mut planned = vec![];
    let mut path = None;
    let result = (|| -> Result<()> {
        let root = store.shelf_path(&args.shelf, true)?;
        path = Some(root.clone());
        let mut snapshot = shelf_snapshot(store, &root)?;
        ensure!(
            snapshot.contains_key(&root.join("bits"))
                || snapshot.contains_key(&root.join("bs.toml")),
            "not a shelf: {} (no bits directory or bs.toml)",
            root.display()
        );
        // Remove the empty bits directory first and settings last. No recursive
        // removal: unexpected contents are always retained, even after preflight.
        for name in ["bits", "SHELF.md", "bs.toml"] {
            let item = root.join(name);
            if snapshot.contains_key(&item) {
                planned.push(item);
            }
        }
        planned.push(root.clone());
        if args.dry_run {
            return Ok(());
        }
        for item in &planned {
            before_remove(item);
            ensure!(
                shelf_snapshot(store, &root)? == snapshot,
                "shelf changed during deletion; remaining contents preserved"
            );
            store.safe(item)?;
            match snapshot.get(item).and_then(Option::as_deref) {
                Some(bytes) => {
                    if let Err(error) = remove_locked(store, item, bytes, false) {
                        if error
                            .downcast_ref::<crate::filesystem::RemovalUncertain>()
                            .is_some()
                        {
                            removed.push(item.clone());
                        }
                        return Err(error);
                    }
                    removed.push(item.clone());
                }
                None => {
                    fs::remove_dir(item)?;
                    removed.push(item.clone());
                    crate::filesystem::sync_parent(item)?;
                }
            }
            snapshot.remove(item);
        }
        Ok(())
    })();
    let failed = result.is_err();
    let status = match &result {
        Ok(()) if args.dry_run => "would_remove",
        Ok(()) => "removed",
        Err(_) if removed.is_empty() => "skipped",
        Err(_) => "partial",
    };
    let mut row =
        json!({"name":args.shelf,"path":path,"status":status,"planned":planned,"removed":removed});
    if let Err(error) = result {
        row["error"] = json!(format!("{error:#}"));
    }
    Ok((row, failed))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guarded_delete_refuses_changed_bytes_and_stops_after_durability_failure() {
        let temp = tempfile::tempdir().unwrap();
        let store = Store {
            config: crate::config::Config {
                store: temp.path().into(),
                editor: None,
                aliases: Default::default(),
            },
        };
        fs::create_dir_all(temp.path().join("notes/bits")).unwrap();
        let a = store.write_bit("notes/a", "original").unwrap();
        let b = store.write_bit("notes/b", "kept").unwrap();
        fs::write(&a, "changed").unwrap();
        for dry_run in [true, false] {
            assert!(remove_locked(&store, &a, b"original", dry_run).is_err());
            assert_eq!(fs::read(&a).unwrap(), b"changed");
        }
        crate::filesystem::inject(&a, crate::filesystem::Stage::DirectorySync);
        let (rows, failed) = run(
            &store,
            Delete {
                id: vec!["notes/a".into(), "notes/b".into()],
                dry_run: false,
            },
        )
        .unwrap();
        assert!(failed);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["status"], "removed_with_error");
        assert!(!a.exists());
        assert!(b.exists());
    }
    #[test]
    fn shelf_deletion_rechecks_contents_and_reports_partial_progress() {
        for fault in ["added-bit", "changed-guidance", "sync", "remove"] {
            let temp = tempfile::tempdir().unwrap();
            let store = Store {
                config: crate::config::Config {
                    store: temp.path().into(),
                    editor: None,
                    aliases: Default::default(),
                },
            };
            fs::create_dir_all(temp.path().join("notes/bits")).unwrap();
            fs::write(temp.path().join("notes/bs.toml"), "").unwrap();
            fs::write(temp.path().join("notes/SHELF.md"), "original").unwrap();
            let (row, failed) = shelf_with(
                &store,
                crate::cli::ShelfDelete {
                    shelf: "notes".into(),
                    dry_run: false,
                },
                |path| {
                    if path.ends_with("bits") {
                        match fault {
                            "added-bit" => {
                                fs::write(path.join("new.md"), "new").unwrap();
                            }
                            "changed-guidance" => {
                                fs::write(temp.path().join("notes/SHELF.md"), "changed").unwrap();
                            }
                            "sync" => crate::filesystem::inject(
                                path,
                                crate::filesystem::Stage::DirectorySync,
                            ),
                            _ => (),
                        }
                    }
                    if fault == "remove" && path.ends_with("SHELF.md") {
                        crate::filesystem::inject(path, crate::filesystem::Stage::Remove);
                    }
                },
            )
            .unwrap();
            assert!(failed, "{fault}");
            assert_eq!(
                row["status"],
                if fault == "added-bit" || fault == "changed-guidance" {
                    "skipped"
                } else {
                    "partial"
                }
            );
            assert!(temp.path().join("notes/bs.toml").exists());
            assert!(temp.path().join("notes/SHELF.md").exists());
            if fault == "added-bit" {
                assert!(temp.path().join("notes/bits/new.md").exists());
            }
            if fault == "sync" || fault == "remove" {
                assert_eq!(row["removed"].as_array().unwrap().len(), 1);
                assert!(!temp.path().join("notes/bits").exists());
            }
        }
    }
}
