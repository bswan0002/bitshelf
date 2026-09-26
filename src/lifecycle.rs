//! Markdown-authoritative command-local timestamp finalization.
use crate::{bit, store::Store};
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use std::{fs, io::Write, path::Path};

pub fn render(map: &crate::metadata::Mapping, body: &str) -> Result<String> {
    Ok(format!("---\n{}---\n{body}", serde_saphyr::to_string(map)?))
}
/// Compare only the bytes and values observed by this command. Dates never
/// come from filesystem times or persistent history.
pub fn finalize(
    original: &str,
    candidate: &str,
    now: DateTime<Utc>,
    editor: bool,
) -> Result<(String, bool)> {
    let (mut before, old_body) = bit::parse(original)?;
    let (mut after, body) = bit::parse(candidate)?;
    // Validate date fields before automatic stamping: invalid imported dates
    // cannot be hidden by an unrelated content mutation.
    for key in ["created", "updated"] {
        if let Some(value) = after.get(key) {
            ensure!(
                bit::timestamp(value).is_some(),
                "invalid reserved field {key}; remove it with bs edit ID --unset {key}, or fix it in an editor"
            );
        }
    }
    // Removing an invalid reserved date is a real change even if nothing else is.
    let repaired = ["created", "updated"].iter().any(|key| {
        before
            .get(*key)
            .is_some_and(|v| bit::timestamp(v).is_none())
            && !after.contains_key(*key)
    });
    let previous_updated = before.get("updated").and_then(bit::timestamp);
    let explicit_updated = editor && before.get("updated") != after.get("updated");
    before.remove("created");
    before.remove("updated");
    let mut user_after = after.clone();
    user_after.remove("created");
    user_after.remove("updated");
    let changed = before != user_after || old_body != body || repaired;
    if changed && !explicit_updated && previous_updated.is_none_or(|old| now > old) {
        after.insert(
            "updated".into(),
            now.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
                .into(),
        );
        return Ok((render(&after, body)?, true));
    }
    Ok((candidate.to_owned(), changed))
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
    if before != after {
        crate::filesystem::publish(&path, after.as_bytes(), Some(before.as_bytes()), None)?;
    } else {
        crate::filesystem::unchanged(&path, before.as_bytes())?;
    }
    Ok(())
}

pub fn edit(store: &Store, args: crate::cli::Edit, json: bool) -> Result<serde_json::Value> {
    crate::usage_check(
        !(args.file.is_some() && args.stdin),
        "--file and --stdin are mutually exclusive",
    )?;
    let interactive = args.file.is_none()
        && !args.stdin
        && args.title.is_none()
        && args.tags.is_none()
        && args.set.is_empty()
        && args.set_json.is_empty()
        && args.unset.is_empty();
    crate::usage_check(
        !(interactive && json),
        "bs edit --json requires --file, --stdin, --title, --tags, --set, --set-json or --unset (no editor)",
    )?;
    let path = store.existing_bit(&args.id)?;
    let original =
        fs::read_to_string(&path).with_context(|| format!("cannot read bit {}", args.id))?;
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
        crate::metadata::Mutation {
            title: args.title,
            tags: args.tags,
            set: args.set,
            set_json: args.set_json,
            unset: args.unset,
        }
        .apply(&mut map)?;
        render(&map, &body)?
    };
    let operation = (|| -> Result<serde_json::Value> {
        let _lock = crate::locking::Lock::acquire(store)?;
        let (next, changed) = finalize(&original, &candidate, Utc::now(), interactive)?;
        validate(store, &args.id, &next)?;
        let next = if !interactive && !changed {
            original.clone()
        } else {
            next
        };
        replace(store, &args.id, &original, &next)?;
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
    let timestamp = now.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true);
    map.insert("created".into(), timestamp.clone().into());
    map.insert("updated".into(), timestamp.into());
    render(&map, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }
    #[test]
    fn imported_missing_dates_and_exact_noops() {
        for raw in [
            "body\r\n",
            "---\ncreated: '2020-01-01T03:00:00+03:00'\n---\nbody",
        ] {
            assert_eq!(
                finalize(raw, raw, now(), false).unwrap(),
                (raw.into(), false)
            );
            let (changed, yes) = finalize(raw, &format!("{raw}more"), now(), false).unwrap();
            assert!(yes);
            let (before, _) = bit::parse(raw).unwrap();
            let (after, body) = bit::parse(&changed).unwrap();
            assert_eq!(before.get("created"), after.get("created"));
            assert!(after.contains_key("updated"));
            assert!(body.ends_with("more"));
        }
    }
    #[test]
    fn explicit_dates_clock_clamp_and_invalid_imports() {
        let original =
            "---\ncreated: '2020-01-01T00:00:00Z'\nupdated: '2099-01-01T03:00:00+03:00'\n---\nbody";
        let changed = format!("{original}more");
        assert_eq!(
            finalize(original, &changed, now(), false).unwrap(),
            (changed, true)
        );
        for candidate in ["---\nupdated: '2001-01-01T00:00:00Z'\n---\nnew", "new"] {
            assert_eq!(
                finalize(original, candidate, now(), true).unwrap(),
                (candidate.into(), true)
            );
        }
        let bad = "---\nupdated: bad\n---\nbody";
        assert!(finalize(bad, &format!("{bad}more"), now(), false).is_err());
        // Removing the invalid date in an editor is a repair, not a no-op.
        assert_eq!(
            finalize(bad, "body", now(), true).unwrap(),
            ("body".into(), true)
        );
        let (repaired, changed) = finalize(bad, "body", now(), false).unwrap();
        assert!(changed);
        assert!(
            bit::parse(&repaired).unwrap().0["updated"]
                .as_str()
                .is_some_and(|s| s.starts_with("2026-01-01"))
        );
    }
    #[test]
    fn semantic_comments_order_and_date_only_edits() {
        let original = "---\ncustom: {b: 2, a: 1}\n---\nbody";
        let candidate =
            "---\ncustom: {a: 1, b: 2} # comment\ncreated: '2020-01-01T00:00:00Z'\n---\nbody";
        assert_eq!(
            finalize(original, candidate, now(), true).unwrap(),
            (candidate.into(), false)
        );
    }
}
