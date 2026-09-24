//! A move publishes a validated destination before removing the source.
use crate::{bit, cli::Move, lifecycle, store::Store};
use anyhow::{Context, Result, ensure};
use chrono::Utc;
use serde_json::{Value, json};
use std::{fs, io::Write};

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
    let mut state = lifecycle::State::load(store, !args.dry_run)?;
    let before =
        fs::read_to_string(&source).with_context(|| format!("cannot read bit {}", args.id))?;
    let now = Utc::now();
    let (raw, entry, _) = lifecycle::reconcile(&before, state.get(&args.id), now)?;
    let after = if args.set.is_empty() {
        raw
    } else {
        let (mut metadata, body) = bit::parse(&raw)?;
        for assignment in &args.set {
            let (key, value) = assignment
                .split_once('=')
                .context("--set requires KEY=VALUE")?;
            ensure!(
                !key.trim().is_empty() && !key.chars().any(char::is_control),
                "invalid metadata key"
            );
            ensure!(
                !["created", "updated"].contains(&key),
                "{key} is reserved and managed automatically"
            );
            metadata.insert(key.into(), value.into());
        }
        lifecycle::render(&metadata, body)?
    };
    let (after, entry, _) = lifecycle::reconcile(&after, Some(&entry), now)?;
    lifecycle::validate(store, &id, &after)?;
    let result = json!({"from":args.id,"id":id,"path":destination,"dry_run":args.dry_run});
    if args.dry_run {
        return Ok(result);
    }

    // Copy into a destination-local temporary file (also supports different mounts).
    // Publish without clobbering, then remove the source. A crash can leave both
    // copies, but never deliberately removes the only copy of a bit.
    let mut tmp = tempfile::NamedTempFile::new_in(destination.parent().unwrap())?;
    tmp.as_file()
        .set_permissions(fs::metadata(&source)?.permissions())?;
    tmp.write_all(after.as_bytes())?;
    tmp.as_file().sync_all()?;
    store.safe(&destination)?;
    store.safe(&source)?;
    ensure!(
        fs::read_to_string(&source)? == before,
        "source changed during move; nothing moved"
    );
    tmp.persist_noclobber(&destination)
        .with_context(|| format!("cannot create {id}; source preserved"))?;
    let remove = (|| -> Result<()> {
        store.safe(&source)?;
        ensure!(
            fs::read_to_string(&source)? == before,
            "source changed during move"
        );
        fs::remove_file(&source)?;
        Ok(())
    })();
    if let Err(error) = remove {
        // Do not remove a destination externally changed since publication.
        let rollback = (|| -> Result<()> {
            store.safe(&destination)?;
            ensure!(
                fs::read_to_string(&destination)? == after,
                "destination changed during rollback"
            );
            fs::remove_file(&destination)?;
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
    state.forget(&args.id);
    state.set(&id, entry);
    state.save(store).with_context(|| format!("bit moved to {id}, but tracking state could not be saved; fix the error and run bs sync (do not repeat the move)"))?;
    Ok(result)
}
