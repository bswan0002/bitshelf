use crate::{bit::Bit, config::ShelfConfig};
use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq)]
pub enum Decision {
    Keep,
    Remove,
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
    Decision::Remove
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
        assert_eq!(decide(&bit, &temporary, now), Decision::Remove);
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
    use anyhow::Context;
    use serde_json::json;
    use std::fs;
    let _lock = if c.dry_run {
        None
    } else {
        Some(crate::locking::Lock::acquire(store)?)
    };
    let now = Utc::now();
    let bits = store.bits(c.shelf.as_deref())?;
    let mut results = vec![];
    let mut failed = false;
    for b in bits {
        let shelf = b.id.split('/').next().unwrap();
        match decide(&b, &store.settings(shelf)?, now) {
            Decision::Keep => continue,
            Decision::Skip(error) => {
                eprintln!("warning: {}: {error}; skipped", b.id);
                results.push(json!({"id":b.id,"path":b.path,"status":"skipped","error":error}));
                failed = true;
                continue;
            }
            Decision::Remove => (),
        }
        store.safe(&b.path)?;
        if !c.dry_run {
            fs::remove_file(&b.path)
                .with_context(|| format!("cannot remove {}", b.path.display()))?;
        }
        results.push(json!({"id":b.id,"path":b.path,"status":if c.dry_run {"would_remove"} else {"removed"}}));
    }
    Ok((results, failed))
}
