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
