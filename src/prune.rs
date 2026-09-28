use crate::{bit::Bit, config::ShelfConfig};
use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq)]
pub enum Decision {
    Keep,
    Expire,
    Skip(&'static str),
}

/// Pure policy; callers perform guarded filesystem operations separately.
pub fn decide(bit: &Bit, shelf: &ShelfConfig, now: DateTime<Utc>) -> Decision {
    if shelf.retention.is_none() {
        return Decision::Keep;
    }
    let Some(expires) = bit.expires else {
        return Decision::Skip("missing or invalid explicit expires");
    };
    if expires > now {
        return Decision::Keep;
    }
    if !bit.errors.is_empty() {
        return Decision::Skip("invalid metadata");
    }
    Decision::Expire
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_boundary_permanent_and_invalid() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let temporary = ShelfConfig {
            retention: Some("1d".into()),
            ..Default::default()
        };
        let bit = crate::bit::inspect(
            "tmp/boundary".into(),
            "/store/tmp/boundary.md".into(),
            "---\nexpires: '2026-01-01T00:00:00Z'\n---\nbody",
            &temporary,
        );
        assert_eq!(
            decide(&bit, &temporary, now - chrono::Duration::seconds(1)),
            Decision::Keep
        );
        assert_eq!(decide(&bit, &temporary, now), Decision::Expire);
        assert_eq!(decide(&bit, &ShelfConfig::default(), now), Decision::Keep);
        let bad = crate::bit::inspect(
            "tmp/bad".into(),
            "/store/tmp/bad.md".into(),
            "---\ntitle: []\nexpires: '2000-01-01T00:00:00Z'\n---\n",
            &temporary,
        );
        assert_eq!(
            decide(&bad, &temporary, now),
            Decision::Skip("invalid metadata")
        );
    }
}

pub fn run(
    store: &crate::store::Store,
    c: crate::cli::Prune,
) -> anyhow::Result<(Vec<serde_json::Value>, bool)> {
    run_with(store, c, |_| {})
}
fn run_with(
    store: &crate::store::Store,
    c: crate::cli::Prune,
    mut before_remove: impl FnMut(&Bit),
) -> anyhow::Result<(Vec<serde_json::Value>, bool)> {
    use serde_json::json;
    let _lock = if c.dry_run {
        None
    } else {
        Some(crate::locking::Lock::acquire(store)?)
    };
    let now = Utc::now();
    let bits = store.bits(c.shelf.as_deref())?;
    let mut settings = std::collections::BTreeMap::new();
    // Per-bit metadata errors are decided below with the bit itself (one row
    // each). Unreadable entries on permanent shelves are not prune candidates;
    // everything else (unknown retention, unreadable scopes) stays skipped.
    let discovered: std::collections::BTreeSet<_> = bits
        .results
        .iter()
        .map(|b| b.path.to_string_lossy().into_owned())
        .collect();
    let mut results = vec![];
    for e in bits.errors {
        if discovered.contains(&e.path) {
            continue;
        }
        let path = std::path::Path::new(&e.path);
        let permanent = path
            .strip_prefix(&store.config.store)
            .ok()
            .map(|rel| rel.components().collect::<Vec<_>>())
            .filter(|parts| parts.len() == 3 && parts[1].as_os_str() == "bits")
            .and_then(|parts| parts[0].as_os_str().to_str().map(str::to_owned))
            .is_some_and(|shelf| {
                let cfg = settings
                    .entry(shelf.clone())
                    .or_insert_with(|| store.settings(&shelf).map_err(|e| format!("{e:#}")));
                cfg.as_ref().is_ok_and(|cfg| cfg.retention.is_none())
            });
        if !permanent {
            results.push(json!({"id":null,"path":e.path,"status":"skipped","error":e.error}));
        }
    }
    let mut failed = !results.is_empty();
    for b in bits.results {
        let mut destination = None;
        let operation = (|| -> anyhow::Result<Option<&str>> {
            let shelf = b.id.split('/').next().unwrap();
            let cfg = settings
                .entry(shelf.to_owned())
                .or_insert_with(|| store.settings(shelf).map_err(|e| format!("{e:#}")));
            let cfg = cfg.as_ref().map_err(|e| anyhow::anyhow!(e.clone()))?;
            match decide(&b, cfg, now) {
                Decision::Keep => return Ok(None),
                Decision::Skip(error) => anyhow::bail!(error),
                Decision::Expire => (),
            }
            let (_, name) = b.id.split_once('/').unwrap();
            destination = cfg
                .on_expire
                .as_ref()
                .map(|action| action.destination(shelf, name))
                .transpose()?
                .flatten();
            before_remove(&b);
            store.safe(&b.path)?;
            crate::filesystem::unchanged(&b.path, b.raw.as_bytes())?;
            // Any policy change invalidates the planned operation, not just
            // disabling retention. Never delete after a failed move.
            anyhow::ensure!(
                &store.settings(shelf)? == cfg,
                "shelf settings (retention or on_expire) changed during prune"
            );
            if let Some(destination) = &destination {
                crate::moving::run_locked(
                    store,
                    crate::cli::Move {
                        id: b.id.clone(),
                        destination: destination.clone(),
                        title: None,
                        tags: None,
                        set: vec![],
                        set_json: vec![],
                        unset: vec![],
                        dry_run: c.dry_run,
                    },
                    Some(b.raw.as_bytes()),
                )?;
                Ok(Some(if c.dry_run { "would_move" } else { "moved" }))
            } else {
                crate::deleting::remove_locked(store, &b.path, b.raw.as_bytes(), c.dry_run)?;
                Ok(Some(if c.dry_run { "would_remove" } else { "removed" }))
            }
        })();
        match operation {
            Ok(None) => (),
            Ok(Some(status)) => {
                let mut row = json!({"id":b.id,"path":b.path,"status":status});
                if let Some(destination) = destination {
                    row["destination"] = json!(destination);
                }
                results.push(row);
            }
            Err(error) => {
                failed = true;
                let move_error = error.downcast_ref::<crate::moving::UncertainMove>();
                let removed = error
                    .downcast_ref::<crate::filesystem::RemovalUncertain>()
                    .is_some();
                let status = match move_error {
                    Some(crate::moving::UncertainMove::SourceRemoved) => "moved_with_error",
                    Some(_) => "move_uncertain",
                    None if removed => "removed_with_error",
                    None => "skipped",
                };
                let mut row =
                    json!({"id":b.id,"path":b.path,"status":status,"error":format!("{error:#}")});
                if let Some(destination) = destination {
                    row["destination"] = json!(destination);
                }
                results.push(row);
                // After uncertain publication/durability, leave later candidates
                // untouched so the user can inspect before any further work.
                if move_error.is_some() || removed {
                    break;
                }
            }
        }
    }
    Ok((results, failed))
}

#[cfg(test)]
mod operation_tests {
    use super::*;
    #[test]
    fn changed_expiry_body_and_partial_failure_are_retained() {
        use std::fs;
        let temp = tempfile::tempdir().unwrap();
        let store = crate::store::Store {
            config: crate::config::Config {
                store: temp.path().into(),
                editor: None,
                aliases: Default::default(),
            },
        };
        fs::create_dir_all(temp.path().join("tmp/bits")).unwrap();
        fs::write(temp.path().join("tmp/bs.toml"), "retention = '1d'").unwrap();
        let raw = "---\nexpires: '2000-01-01T00:00:00Z'\n---\nbody";
        for name in ["a", "body", "expiry", "remove-expiry", "failure"] {
            store.write_bit(&format!("tmp/{name}"), raw).unwrap();
        }
        let (rows, failed) = run_with(
            &store,
            crate::cli::Prune {
                shelf: None,
                dry_run: false,
            },
            |b| match b.id.as_str() {
                "tmp/body" => fs::write(&b.path, format!("{raw}changed")).unwrap(),
                "tmp/expiry" => fs::write(&b.path, raw.replace("2000", "2099")).unwrap(),
                "tmp/remove-expiry" => fs::write(&b.path, "body").unwrap(),
                "tmp/failure" => {
                    crate::filesystem::inject(&b.path, crate::filesystem::Stage::Remove)
                }
                _ => (),
            },
        )
        .unwrap();
        assert!(failed);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0]["status"], "removed");
        for row in &rows[1..] {
            assert_eq!(row["status"], "skipped");
            assert!(std::path::Path::new(row["path"].as_str().unwrap()).exists());
        }
    }
    fn fixture() -> (tempfile::TempDir, crate::store::Store) {
        let temp = tempfile::tempdir().unwrap();
        let store = crate::store::Store {
            config: crate::config::Config {
                store: temp.path().into(),
                editor: None,
                aliases: Default::default(),
            },
        };
        for shelf in ["tmp", "archive"] {
            std::fs::create_dir_all(temp.path().join(format!("{shelf}/bits"))).unwrap();
        }
        std::fs::write(
            temp.path().join("tmp/bs.toml"),
            "retention = '1d'\non_expire = { move = 'archive/{name}' }",
        )
        .unwrap();
        for name in ["a", "b"] {
            store
                .write_bit(
                    &format!("tmp/{name}"),
                    "---\nexpires: '2000-01-01T00:00:00Z'\n---\nbody",
                )
                .unwrap();
        }
        (temp, store)
    }

    #[test]
    fn expiry_move_rechecks_snapshot_and_policy() {
        for change in ["body", "action", "retention"] {
            for dry_run in [false, true] {
                let (_temp, store) = fixture();
                let (rows, failed) =
                    run_with(
                        &store,
                        crate::cli::Prune {
                            shelf: Some("tmp".into()),
                            dry_run,
                        },
                        |bit| {
                            if bit.id != "tmp/a" {
                                return;
                            }
                            match change {
                                "body" => std::fs::write(&bit.path, format!("{}changed", bit.raw))
                                    .unwrap(),
                                "action" => std::fs::write(
                                    store.config.store.join("tmp/bs.toml"),
                                    "retention = '1d'\non_expire = 'delete'",
                                )
                                .unwrap(),
                                _ => std::fs::write(store.config.store.join("tmp/bs.toml"), "")
                                    .unwrap(),
                            }
                        },
                    )
                    .unwrap();
                assert!(failed);
                assert_eq!(rows[0]["status"], "skipped");
                assert!(store.existing_bit("tmp/a").is_ok());
                assert!(!store.bit_path("archive/a").unwrap().exists());
            }
        }
    }

    #[test]
    fn expiry_move_faults_preserve_sources_or_report_uncertainty_and_stop() {
        use crate::filesystem::{self, Stage};
        for (target, stage, expected) in [
            ("tmp/a", Stage::Remove, "skipped"),
            ("tmp/a", Stage::DirectorySync, "moved_with_error"),
            ("archive/a", Stage::DirectorySync, "move_uncertain"),
            ("archive/a", Stage::Publish, "move_uncertain"),
        ] {
            let (_temp, store) = fixture();
            let (rows, failed) = run_with(
                &store,
                crate::cli::Prune {
                    shelf: Some("tmp".into()),
                    dry_run: false,
                },
                |b| {
                    if b.id == "tmp/a" {
                        filesystem::inject(&store.bit_path(target).unwrap(), stage);
                    }
                },
            )
            .unwrap();
            assert!(failed);
            assert_eq!(rows[0]["status"], expected);
            assert_eq!(rows[0]["destination"], "archive/a");
            if expected == "skipped" {
                assert!(store.existing_bit("tmp/a").is_ok());
                assert!(!store.bit_path("archive/a").unwrap().exists());
                assert_eq!(rows[1]["status"], "moved");
            } else {
                assert_eq!(rows.len(), 1);
                assert!(store.existing_bit("tmp/b").is_ok());
                if expected == "moved_with_error" {
                    assert!(!store.bit_path("tmp/a").unwrap().exists());
                    assert!(store.existing_bit("archive/a").is_ok());
                } else {
                    assert!(store.existing_bit("tmp/a").is_ok());
                }
            }
        }
    }

    #[test]
    fn prune_removal_durability_failure_stops_later_candidates() {
        let (_temp, store) = fixture();
        std::fs::write(store.config.store.join("tmp/bs.toml"), "retention = '1d'").unwrap();
        let (rows, failed) = run_with(
            &store,
            crate::cli::Prune {
                shelf: Some("tmp".into()),
                dry_run: false,
            },
            |b| {
                if b.id == "tmp/a" {
                    crate::filesystem::inject(&b.path, crate::filesystem::Stage::DirectorySync);
                }
            },
        )
        .unwrap();
        assert!(failed);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["status"], "removed_with_error");
        assert!(!store.bit_path("tmp/a").unwrap().exists());
        assert!(store.existing_bit("tmp/b").is_ok());
    }
}
