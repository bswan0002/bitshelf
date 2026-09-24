//! Lifecycle timestamps and store-local change-detection state.
use crate::{bit, store::Store};
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    hash: String,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}
#[derive(Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Data {
    entries: BTreeMap<String, Entry>,
}
pub struct State {
    path: PathBuf,
    original: String,
    data: Data,
    _lock: Option<Lock>,
}
struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
impl State {
    pub fn load(store: &Store, write: bool) -> Result<Self> {
        ensure!(
            store.config.store.is_dir(),
            "store does not exist: {}",
            store.config.store.display()
        );
        let dir = store.config.store.join(".bitshelf");
        store.safe(&dir)?;
        let lock = if write {
            fs::create_dir_all(&dir)?;
            let path = dir.join("state.lock");
            store.safe(&path)?;
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path)
                .with_context(|| format!("cannot lock {}; another add/edit/move/sync/prune may be running. If a process crashed, remove this lock only after confirming none is running", path.display()))?;
            let lock = Lock(path);
            writeln!(file, "{}", std::process::id())?;
            Some(lock)
        } else {
            None
        };
        let path = dir.join("state.json");
        store.safe(&path)?;
        let source = match fs::read_to_string(&path) {
            Ok(s) => Some(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let data = match source.as_deref() {
            None => Data::default(),
            Some(text) => serde_json::from_str(text).context("invalid lifecycle state; restore .bitshelf/state.json from backup or remove it and run bs sync to re-baseline")?,
        };
        let original = source.unwrap_or_default();
        let mut state = Self {
            path,
            original,
            data,
            _lock: lock,
        };
        state.forget_missing(store)?;
        Ok(state)
    }
    // Absence ends an identifier's history. Do not infer deletion from a
    // permissions error or a refused symlink, and never follow a stored path.
    fn forget_missing(&mut self, store: &Store) -> Result<()> {
        let mut missing = vec![];
        for id in self.data.entries.keys() {
            let (shelf, slug) = id
                .split_once('/')
                .context("invalid identifier in lifecycle state")?;
            crate::config::name(shelf)?;
            crate::config::name(slug)?;
            let path = store
                .config
                .store
                .join(shelf)
                .join("bits")
                .join(format!("{slug}.md"));
            if store.safe(&path).is_err() {
                continue;
            }
            match fs::symlink_metadata(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => missing.push(id.clone()),
                Err(e) => return Err(e).with_context(|| format!("checking tracked bit {id}")),
                Ok(meta) if !meta.is_file() => missing.push(id.clone()),
                Ok(_) => (),
            }
        }
        for id in missing {
            self.forget(&id);
        }
        Ok(())
    }
    pub fn forget(&mut self, id: &str) {
        self.data.entries.remove(id);
    }
    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.data.entries.get(id)
    }
    pub fn remember(&mut self, id: &str, raw: &str) -> Result<()> {
        self.data
            .entries
            .insert(id.into(), baseline(raw, Utc::now())?);
        Ok(())
    }
    pub fn set(&mut self, id: &str, entry: Entry) {
        self.data.entries.insert(id.into(), entry);
    }
    pub fn save_after_bit(&self, store: &Store, id: &str) -> Result<()> {
        self.save(store).with_context(|| format!(
            "bit {id} was saved, but tracking state could not be saved; fix the error and run bs sync (do not repeat bs add)",
        ))
    }
    pub fn save(&self, store: &Store) -> Result<()> {
        let text = serde_json::to_string_pretty(&self.data)?;
        if text != self.original {
            store.safe(&self.path)?;
            atomic_write(&self.path, text.as_bytes())?;
        }
        Ok(())
    }
}

pub fn render(map: &serde_yaml::Mapping, body: &str) -> Result<String> {
    Ok(format!("---\n{}---\n{body}", serde_yaml::to_string(map)?))
}
fn hash(raw: &str) -> Result<String> {
    let (mut map, body) = bit::parse(raw)?;
    map.remove("created");
    map.remove("updated");
    // JSON object keys are sorted, so YAML key order/comments are not edits.
    let metadata = serde_json::to_value(map)?;
    let bytes = serde_json::to_vec(&(metadata, body))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn baseline(raw: &str, now: DateTime<Utc>) -> Result<Entry> {
    let (map, _) = bit::parse(raw)?;
    let date = |key: &str| -> Result<Option<DateTime<Utc>>> {
        map.get(key)
            .map(|v| bit::timestamp(v).with_context(|| format!("invalid reserved field {key}")))
            .transpose()
    };
    Ok(Entry {
        hash: hash(raw)?,
        created: date("created")?.unwrap_or(now),
        updated: date("updated")?.unwrap_or(now),
    })
}

/// Compare against the last known content, restoring reserved timestamps from
/// state. Untracked files preserve valid dates and use discovery time if absent.
pub fn reconcile(
    raw: &str,
    previous: Option<&Entry>,
    now: DateTime<Utc>,
) -> Result<(String, Entry, bool)> {
    let mut entry = match previous {
        Some(entry) => entry.clone(),
        None => baseline(raw, now)?,
    };
    let next_hash = hash(raw)?;
    let changed = previous.is_some() && entry.hash != next_hash;
    if changed {
        entry.updated = now.max(entry.updated);
    }
    entry.hash = next_hash;
    let (mut map, body) = bit::parse(raw)?;
    let mut rewritten = false;
    for (key, value) in [("created", entry.created), ("updated", entry.updated)] {
        let value =
            serde_yaml::Value::String(value.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true));
        // Retain existing formatting if it represents the same instant.
        if map.get(key).and_then(bit::timestamp) != bit::timestamp(&value) {
            map.insert(key.into(), value);
            rewritten = true;
        }
    }
    Ok((
        if rewritten {
            render(&map, body)?
        } else {
            raw.into()
        },
        entry,
        changed,
    ))
}

pub fn validate(store: &Store, id: &str, raw: &str) -> Result<()> {
    let shelf = id.split('/').next().unwrap();
    let checked = bit::inspect(id.into(), store.bit_path(id)?, raw, &store.settings(shelf)?);
    ensure!(
        checked.errors.is_empty(),
        "invalid metadata for {id}: {}",
        checked.errors.join(", ")
    );
    Ok(())
}
pub fn replace(store: &Store, id: &str, before: &str, after: &str) -> Result<()> {
    let path = store.bit_path(id)?;
    ensure!(
        fs::read_to_string(&path)? == before,
        "{id} changed during operation; rerun (no changes overwritten)"
    );
    if before != after {
        atomic_write(&path, after.as_bytes())?;
    }
    Ok(())
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut tmp = tempfile::NamedTempFile::new_in(path.parent().context("missing parent")?)?;
    if let Ok(meta) = fs::metadata(path) {
        tmp.as_file().set_permissions(meta.permissions())?;
    }
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
}

#[derive(Serialize)]
pub struct SyncResult {
    pub id: String,
    pub changed: bool,
    pub metadata_changed: bool,
    pub baselined: bool,
    pub error: Option<String>,
}
pub fn sync(store: &Store, shelf: Option<&str>, dry_run: bool) -> Result<Vec<SyncResult>> {
    let mut state = State::load(store, !dry_run)?;
    let mut results = vec![];
    for b in store.bits_for_sync(shelf)? {
        let mut result = SyncResult {
            id: b.id.clone(),
            changed: false,
            metadata_changed: false,
            baselined: false,
            error: None,
        };
        let operation = (|| -> Result<()> {
            let raw = fs::read_to_string(store.bit_path(&b.id)?)?;
            let (next, entry, changed) = reconcile(&raw, state.get(&b.id), Utc::now())?;
            // Timestamp reconciliation is not authoring validation. Preserve
            // incomplete user metadata; `bs validate` reports those issues.
            result.changed = changed;
            result.metadata_changed = raw != next;
            result.baselined = state.get(&b.id).is_none();
            if !dry_run {
                replace(store, &b.id, &raw, &next)?;
                state.set(&b.id, entry);
            }
            Ok(())
        })();
        if let Err(e) = operation {
            result.error = Some(format!("{e:#}"));
        }
        results.push(result);
    }
    if !dry_run {
        state.save(store).context(
            "sync may have updated bit files, but tracking state could not be saved; fix the error and rerun bs sync",
        )?;
    }
    Ok(results)
}

pub fn edit(store: &Store, args: crate::cli::Edit, json: bool) -> Result<serde_json::Value> {
    crate::usage_check(
        !(args.file.is_some() && args.stdin),
        "--file and --stdin are mutually exclusive",
    )?;
    let interactive =
        args.file.is_none() && !args.stdin && args.title.is_none() && args.tags.is_none();
    crate::usage_check(
        !(interactive && json),
        "bs edit --json requires --file, --stdin, --title or --tags (no editor)",
    )?;
    let path = store.bit_path(&args.id)?;
    let original = fs::read_to_string(&path)?;
    // Record discovery time before opening the editor, without holding a lock.
    let initial = State::load(store, false)?
        .get(&args.id)
        .cloned()
        .map(Ok)
        .unwrap_or_else(|| baseline(&original, Utc::now()))?;
    let mut draft = None;
    let candidate = if interactive {
        let mut file = tempfile::Builder::new()
            .prefix(".edit-")
            .suffix(".md")
            .tempfile_in(path.parent().unwrap())?;
        file.write_all(original.as_bytes())?;
        let (_, draft_path) = file.keep()?;
        eprintln!("Draft: {}", draft_path.display());
        crate::editor::launch(
            &store.config,
            std::slice::from_ref(&draft_path),
            false,
            crate::editor::Purpose::Edit,
        )
        .with_context(|| format!("edit failed; draft preserved at {}", draft_path.display()))?;
        store
            .safe(&draft_path)
            .with_context(|| format!("unsafe draft; recovery path: {}", draft_path.display()))?;
        let raw = fs::read_to_string(&draft_path).with_context(|| {
            format!("cannot read draft; recovery path: {}", draft_path.display())
        })?;
        draft = Some(draft_path);
        raw
    } else {
        let (mut map, body) = bit::parse(&original)?;
        let body =
            crate::input::body(args.file.as_deref(), args.stdin)?.unwrap_or_else(|| body.into());
        if let Some(title) = args.title {
            map.insert("title".into(), title.into());
        }
        if let Some(tags) = args.tags {
            map.insert(
                "tags".into(),
                serde_yaml::to_value(
                    tags.split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>(),
                )?,
            );
        }
        render(&map, &body)?
    };
    let operation = (|| -> Result<serde_json::Value> {
        // Editors can stay open indefinitely. Lock and reload only for commit,
        // preserving unrelated changes made by other commands in the meantime.
        let mut state = State::load(store, true)?;
        let previous = state.get(&args.id).unwrap_or(&initial);
        let (next, entry, changed) = reconcile(&candidate, Some(previous), Utc::now())?;
        validate(store, &args.id, &next)?;
        // Avoid YAML formatting churn for a no-op CLI edit.
        let next = if !interactive && !changed && hash(&original)? == hash(&next)? {
            reconcile(&original, Some(&entry), Utc::now())?.0
        } else {
            next
        };
        replace(store, &args.id, &original, &next)?;
        state.set(&args.id, entry);
        state.save_after_bit(store, &args.id)?;
        Ok(serde_json::json!({"id":args.id,"path":path,"changed":changed}))
    })();
    match operation {
        Ok(value) => {
            if let Some(path) = draft {
                cleanup_draft(&path);
            }
            Ok(value)
        }
        Err(e) => Err(e).with_context(|| match draft {
            Some(p) => format!("draft preserved at {}", p.display()),
            None => "edit did not complete".into(),
        }),
    }
}

/// Cleanup failure does not turn a successfully saved bit into a failed save.
pub fn cleanup_draft(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        eprintln!(
            "warning: bit saved, but could not remove recovery draft {}: {error}",
            path.display()
        );
    }
}

/// Creation always owns both reserved fields, including interactive drafts.
pub fn new_bit(raw: &str, now: DateTime<Utc>) -> Result<String> {
    let (mut map, body) = bit::parse(raw)?;
    let timestamp = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    map.insert("created".into(), timestamp.clone().into());
    map.insert("updated".into(), timestamp.into());
    render(&map, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }
    #[test]
    fn controlled_clock_hashing_and_reserved_fields() {
        let t0 = at("2020-01-01T00:00:00Z");
        let t1 = at("2021-01-01T00:00:00Z");
        let (initial, first, changed) = reconcile(
            "---\ntitle: Test\ncustom: {b: 2, a: 1}\n---\nbody",
            None,
            t0,
        )
        .unwrap();
        assert!(!changed);
        assert_eq!(first.created, t0);
        assert_eq!(first.updated, t0);
        let (same, second, changed) = reconcile(&initial, Some(&first), t1).unwrap();
        assert!(!changed);
        assert_eq!(same, initial);
        assert_eq!(second, first);
        let raw = "---\ncustom: {a: 1, b: 2} # comment\ntitle: Test\ncreated: BAD\nupdated: 2099-01-01T00:00:00Z\n---\nbody";
        let (fixed, entry, changed) = reconcile(raw, Some(&first), t1).unwrap();
        assert!(!changed);
        assert_eq!(entry, first);
        assert!(!fixed.contains("BAD"));
        let (_, edited, changed) =
            reconcile(&format!("{initial} edited"), Some(&first), t1).unwrap();
        assert!(changed);
        assert_eq!(edited.created, t0);
        assert_eq!(edited.updated, t1);
        let (_, backwards, _) = reconcile(&format!("{initial} again"), Some(&edited), t0).unwrap();
        assert_eq!(backwards.updated, t1);
    }
    #[test]
    fn baseline_preserves_known_dates_and_rejects_invalid_ones() {
        let now = at("2026-01-01T00:00:00Z");
        let raw = "---\ncreated: 2020-01-01T00:00:00Z\nupdated: 2021-01-01T00:00:00Z\n---\nbody";
        let (next, entry, changed) = reconcile(raw, None, now).unwrap();
        assert_eq!(next, raw);
        assert!(!changed);
        assert_eq!(entry.created, at("2020-01-01T00:00:00Z"));
        assert_eq!(entry.updated, at("2021-01-01T00:00:00Z"));
        assert!(reconcile("---\ncreated: yesterday\n---\nbody", None, now).is_err());
    }
    #[test]
    fn state_save_failure_reports_that_the_bit_was_saved() {
        let temp = tempfile::tempdir().unwrap();
        let store = Store {
            config: crate::config::Config {
                aliases: Default::default(),
                store: temp.path().into(),
                editor: None,
            },
        };
        fs::create_dir_all(temp.path().join("notes/bits")).unwrap();
        let mut state = State::load(&store, true).unwrap();
        let raw = new_bit("body", at("2020-01-01T00:00:00Z")).unwrap();
        state.remember("notes/saved", &raw).unwrap();
        let path = store.write_bit("notes/saved", &raw).unwrap();
        // Make the state destination unpublishable after the note was saved.
        fs::create_dir(temp.path().join(".bitshelf/state.json")).unwrap();
        let error = state.save_after_bit(&store, "notes/saved").unwrap_err();
        assert!(error.to_string().contains("bit notes/saved was saved"));
        assert!(error.to_string().contains("run bs sync"));
        assert!(error.to_string().contains("do not repeat bs add"));
        assert_eq!(fs::read_to_string(path).unwrap(), raw);
    }

    #[test]
    fn draft_cleanup_is_best_effort() {
        let temp = tempfile::tempdir().unwrap();
        let draft = temp.path().join(".edit-recovery.md");
        fs::write(&draft, "recovery").unwrap();
        cleanup_draft(&draft);
        assert!(!draft.exists());
        // An unremovable entry warns rather than failing the completed save.
        fs::create_dir(&draft).unwrap();
        cleanup_draft(&draft);
        assert!(draft.is_dir());
    }

    #[test]
    fn equivalent_imported_timezone_representation_is_preserved() {
        let raw = "---\ncreated: 2020-01-01T03:00:00+03:00\nupdated: 2021-01-01T03:00:00+03:00\n---\nbody";
        let (same, entry, changed) = reconcile(raw, None, at("2026-01-01T00:00:00Z")).unwrap();
        assert_eq!(same, raw);
        assert!(!changed);
        let (same, _, changed) = reconcile(raw, Some(&entry), at("2027-01-01T00:00:00Z")).unwrap();
        assert_eq!(same, raw);
        assert!(!changed);
    }
}
