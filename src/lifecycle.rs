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
        let dir = store.config.store.join(".bitshelf");
        store.safe(&dir)?;
        let lock = if write {
            fs::create_dir_all(&dir)?;
            let path = dir.join("state.lock");
            store.safe(&path)?;
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path)
                .with_context(|| format!("cannot lock {}; another add/edit/sync may be running. If a process crashed, remove this lock only after confirming none is running", path.display()))?;
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
        Ok(Self {
            path,
            original,
            data,
            _lock: lock,
        })
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
    let checked = bit::inspect(id.into(), store.bit_path(id)?, raw, &store.settings(shelf));
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
        state.save(store)?;
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
    let mut state = State::load(store, true)?;
    let original = fs::read_to_string(&path)?;
    // Establish a baseline before editing, even for imported/legacy files.
    let previous = match state.get(&args.id) {
        Some(e) => e.clone(),
        None => baseline(&original, Utc::now())?,
    };
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
        store.safe(&draft_path)?;
        let raw = fs::read_to_string(&draft_path)?;
        draft = Some(draft_path);
        raw
    } else {
        let (mut map, body) = bit::parse(&original)?;
        let body = if let Some(file) = args.file {
            fs::read_to_string(file)?
        } else if args.stdin {
            use std::io::Read;
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            text
        } else {
            body.into()
        };
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
        let (next, entry, changed) = reconcile(&candidate, Some(&previous), Utc::now())?;
        validate(store, &args.id, &next)?;
        // Avoid YAML formatting churn for a no-op CLI edit.
        let next = if !interactive && !changed && hash(&original)? == hash(&next)? {
            reconcile(&original, Some(&entry), Utc::now())?.0
        } else {
            next
        };
        replace(store, &args.id, &original, &next)?;
        state.set(&args.id, entry);
        state.save(store)?;
        Ok(serde_json::json!({"id":args.id,"path":path,"changed":changed}))
    })();
    match operation {
        Ok(value) => {
            if let Some(path) = draft {
                fs::remove_file(path)?;
            }
            Ok(value)
        }
        Err(e) => Err(e).with_context(|| match draft {
            Some(p) => format!("draft preserved at {}", p.display()),
            None => "edit failed".into(),
        }),
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
}
