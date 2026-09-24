use crate::config::ShelfConfig;
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::path::PathBuf;

#[derive(Serialize)]
pub struct Bit {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub errors: Vec<String>,
    #[serde(skip)]
    pub body: String,
    #[serde(skip)]
    pub expires: Option<DateTime<Utc>>,
}
pub fn parse(raw: &str) -> Result<(Mapping, &str)> {
    let first = raw.find('\n').map(|i| &raw[..=i]);
    if !matches!(first, Some("---\n" | "---\r\n")) {
        return Ok((Mapping::new(), raw));
    }
    let start = first.unwrap().len();
    let mut offset = start;
    for line in raw[start..].split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            let yaml = &raw[start..offset];
            let value: Value = serde_yaml::from_str(yaml).context("malformed YAML frontmatter")?;
            let map = if value.is_null() {
                Mapping::new()
            } else {
                value
                    .as_mapping()
                    .context("frontmatter must be a mapping")?
                    .clone()
            };
            return Ok((map, &raw[offset + line.len()..]));
        }
        offset += line.len();
    }
    anyhow::bail!("unterminated YAML frontmatter")
}
pub fn timestamp(v: &Value) -> Option<DateTime<Utc>> {
    v.as_str()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
}
pub fn inspect(id: String, path: PathBuf, raw: &str, cfg: &ShelfConfig) -> Bit {
    let mut errors = vec![];
    let (map, body) = match parse(raw) {
        Ok(v) => v,
        Err(e) => {
            errors.push(e.to_string());
            (Mapping::new(), raw)
        }
    };
    for key in &cfg.required {
        if !map.contains_key(Value::String(key.clone())) {
            errors.push(format!("missing required field: {key}"));
        }
    }
    for (key, value) in &map {
        let valid = match key.as_str() {
            Some("title") => value.as_str().is_some_and(|s| !s.trim().is_empty()),
            Some("tags") => value
                .as_sequence()
                .is_some_and(|a| a.iter().all(|v| v.as_str().is_some())),
            Some("created" | "updated" | "expires") => timestamp(value).is_some(),
            _ => true,
        };
        if !valid {
            let field = key.as_str().unwrap_or("metadata field");
            let expected = match field {
                "title" => "a nonempty string",
                "tags" => "a list of strings",
                _ => "an RFC 3339 timestamp, e.g. 2026-01-01T00:00:00Z",
            };
            errors.push(format!("invalid {field}: expected {expected}"));
        }
    }
    let title = map
        .get("title")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| id.split('/').next_back().unwrap())
        .to_string();
    let tags: Vec<String> = map
        .get("tags")
        .and_then(Value::as_sequence)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    // Avoid namespace diagnostics derived from malformed tag metadata.
    let tags_well_formed = map.get("tags").is_none_or(|value| {
        value
            .as_sequence()
            .is_some_and(|items| items.iter().all(|v| v.as_str().is_some()))
    });
    if tags_well_formed {
        for (namespace, rule) in &cfg.tag_rules {
            let prefix = format!("{namespace}:");
            let values: Vec<&str> = tags
                .iter()
                .filter_map(|tag| tag.strip_prefix(&prefix))
                .collect();
            if rule.required && values.is_empty() {
                errors.push(format!(
                    "missing required tag namespace {namespace:?}; expected one of: {}",
                    rule.allowed
                        .iter()
                        .map(|v| format!("{prefix}{v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            for value in values {
                if !rule.allowed.iter().any(|allowed| allowed == value) {
                    errors.push(format!(
                        "disallowed tag {:?}; allowed values for {namespace:?}: {}",
                        format!("{prefix}{value}"),
                        rule.allowed.join(", ")
                    ));
                }
            }
        }
    }
    let expires = map.get("expires").and_then(timestamp);
    Bit {
        id,
        path,
        title,
        tags,
        metadata: serde_json::to_value(&map).unwrap_or(serde_json::Value::Null),
        errors,
        body: body.to_string(),
        expires,
    }
}
pub fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
pub fn create(
    title: &str,
    tags: Option<&str>,
    body: &str,
    cfg: &ShelfConfig,
    now: DateTime<Utc>,
) -> Result<String> {
    ensure!(!title.trim().is_empty(), "title must not be empty");
    let mut map = Mapping::new();
    map.insert("title".into(), title.into());
    map.insert(
        "created".into(),
        now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            .into(),
    );
    map.insert("updated".into(), map["created"].clone());
    if let Some(tags) = tags {
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
    if let Some(r) = &cfg.retention {
        let expires = now
            .checked_add_signed(crate::config::retention(r)?)
            .context("expiration out of range")?;
        map.insert(
            "expires".into(),
            expires
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                .into(),
        );
    }
    Ok(format!("---\n{}---\n{body}", serde_yaml::to_string(&map)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_and_required_tag_namespaces() {
        let mut cfg: ShelfConfig =
            toml::from_str("[tag_rules.project]\nallowed = ['bitshelf']").unwrap();
        let check = |raw: &str, cfg: &ShelfConfig| {
            inspect("notes/test".into(), "/test".into(), raw, cfg).errors
        };
        for raw in [
            "body",
            "---\ntags: []\n---\n",
            "---\ntags: [rust, 'other:anything']\n---\n",
        ] {
            assert!(check(raw, &cfg).is_empty());
        }
        assert!(check("---\ntags: ['project:']\n---\n", &cfg)[0].contains("disallowed tag"));
        cfg.tag_rules.get_mut("project").unwrap().required = true;
        for raw in ["body", "---\ntags: []\n---\n"] {
            assert!(check(raw, &cfg)[0].contains("missing required tag namespace"));
        }
        for raw in [
            "---\ntags: project:bitshelf\n---\n",
            "---\ntags: [4]\n---\n",
        ] {
            assert_eq!(
                check(raw, &cfg),
                ["invalid tags: expected a list of strings"]
            );
        }
        assert!(check("---\ntags: ['project:bitshelf']\n---\n", &cfg).is_empty());
    }
    #[test]
    fn controlled_clock_and_verbatim_body() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let cfg = ShelfConfig {
            retention: Some("14d".into()),
            ..Default::default()
        };
        for body in [
            "",
            "---\nnot metadata",
            "\r\n  exact\r\n",
            "no final newline",
        ] {
            let raw = create("Title: quoted", Some("a,b"), body, &cfg, now).unwrap();
            let (map, recovered) = parse(&raw).unwrap();
            assert_eq!(recovered.as_bytes(), body.as_bytes());
            assert_eq!(map["created"].as_str(), Some("2026-01-01T23:59:59Z"));
            assert_eq!(map["expires"].as_str(), Some("2026-01-15T23:59:59Z"));
        }
    }
    #[test]
    fn crlf_frontmatter_and_validation() {
        let (map, body) = parse("---\r\ntitle: hi\r\n---\r\nbody\r\n").unwrap();
        assert_eq!(map["title"].as_str(), Some("hi"));
        assert_eq!(body, "body\r\n");
        assert!(parse("---\ntitle: hi").is_err());
        assert!(parse("---\n- list\n---\n").is_err());
        let cfg = ShelfConfig {
            required: vec!["title".into()],
            ..Default::default()
        };
        let b = inspect(
            "notes/plain".into(),
            "/store/notes/plain.md".into(),
            "body",
            &cfg,
        );
        assert_eq!(b.title, "plain");
        assert_eq!(b.errors, vec!["missing required field: title"]);
    }
}
