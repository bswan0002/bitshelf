//! A move publishes a validated destination before removing the source.
use crate::{bit, cli::Move, lifecycle, store::Store};
use anyhow::{Context, Result, ensure};
use chrono::Utc;
use serde_json::{Value, json};
use std::fs;

pub fn run(store: &Store, args: Move) -> Result<Value> {
    let source = store.bit_path(&args.id)?;
    let (_, name) = args.id.split_once('/').unwrap();
    let id = if args.destination.contains('/') {
        args.destination.clone()
    } else {
        format!("{}/{name}", args.destination)
    };
    let destination = store.bit_path(&id)?;
    ensure!(id != args.id, "source and destination are the same bit");
    ensure!(
        !destination.try_exists()?,
        "destination {id} already exists; nothing moved"
    );
    let _lock = if args.dry_run {
        None
    } else {
        Some(crate::locking::Lock::acquire(store)?)
    };
    let before =
        fs::read_to_string(&source).with_context(|| format!("cannot read bit {}", args.id))?;
    let now = Utc::now();
    let raw = before.clone();
    let after = if args.set.is_empty()
        && args.set_json.is_empty()
        && args.unset.is_empty()
        && args.title.is_none()
        && args.tags.is_none()
    {
        raw
    } else {
        let (mut metadata, body) = bit::parse(&raw)?;
        crate::metadata::Mutation {
            title: args.title,
            tags: args.tags,
            set: args.set,
            set_json: args.set_json,
            unset: args.unset,
        }
        .apply(&mut metadata)?;
        lifecycle::render(&metadata, body)?
    };
    let (after, _) = lifecycle::finalize(&before, &after, now, false)?;
    lifecycle::validate(store, &id, &after)?;
    let result = json!({"from":args.id,"id":id,"path":destination,"dry_run":args.dry_run});
    if args.dry_run {
        return Ok(result);
    }

    store.safe(&source)?;
    crate::filesystem::unchanged(&source, before.as_bytes())?;
    crate::filesystem::publish(
        &destination,
        after.as_bytes(),
        None,
        Some(fs::metadata(&source)?.permissions()),
    )
    .context(
        "destination publication failed; source preserved; inspect destination before retrying",
    )?;
    let remove = (|| -> Result<()> {
        store.safe(&source)?;
        ensure!(
            fs::read_to_string(&source)? == before,
            "source changed during move"
        );
        crate::filesystem::remove(&source, before.as_bytes())?;
        Ok(())
    })();
    if let Err(error) = remove {
        if !source.try_exists()? {
            return Err(error).context("move destination saved and source removed; destination retained; source-directory durability uncertain");
        }
        // Do not remove a destination externally changed since publication.
        let rollback = (|| -> Result<()> {
            store.safe(&destination)?;
            ensure!(
                fs::read_to_string(&destination)? == after,
                "destination changed during rollback"
            );
            crate::filesystem::remove(&destination, after.as_bytes())?;
            Ok(())
        })();
        if let Err(rollback) = rollback {
            anyhow::bail!(
                "move did not complete: {error:#}; rollback failed: {rollback:#}; inspect both {} and {} before retrying",
                source.display(),
                destination.display()
            );
        }
        return Err(error)
            .context("move did not complete; source preserved, destination rolled back");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_source_removal_rolls_back_but_durability_failure_keeps_destination() {
        use crate::filesystem::{self, Stage};
        for stage in [Stage::Remove, Stage::DirectorySync] {
            let tmp = tempfile::tempdir().unwrap();
            let store = Store {
                config: crate::config::Config {
                    store: tmp.path().into(),
                    editor: None,
                    aliases: Default::default(),
                },
            };
            fs::create_dir_all(tmp.path().join("notes/bits")).unwrap();
            let source = store.write_bit("notes/a", "body").unwrap();
            filesystem::inject(&source, stage);
            let result = run(
                &store,
                Move {
                    id: "notes/a".into(),
                    destination: "notes/b".into(),
                    title: None,
                    tags: None,
                    set_json: vec![],
                    unset: vec![],
                    set: vec![],
                    dry_run: false,
                },
            );
            assert!(result.is_err());
            let destination = store.bit_path("notes/b").unwrap();
            if stage == Stage::Remove {
                assert_eq!(fs::read_to_string(source).unwrap(), "body");
                assert!(!destination.exists());
            } else {
                assert!(!source.exists());
                assert_eq!(fs::read_to_string(destination).unwrap(), "body");
            }
        }
    }
}
