//! One supported value model for JSON, semantic comparison and mutations.
use anyhow::{Result, ensure};
pub type Mapping = serde_json::Map<String, serde_json::Value>;
pub use serde_json::Value;

/// Preflight uses the same maintained scanner as the Serde parser to reject
/// explicit tags, ambiguous integer spellings and overflow before coercion.
pub fn parse(yaml: &str) -> Result<Value> {
    use granit_parser::{ScalarStyle, Scanner, StrInput, TokenType};
    for token in Scanner::new(StrInput::new(yaml)) {
        match token?.into_parts().1 {
            TokenType::Tag(..) => anyhow::bail!("explicit YAML tags are unsupported"),
            TokenType::Scalar(ScalarStyle::Plain, scalar) => {
                let text = scalar.strip_prefix(['+', '-']).unwrap_or(&scalar);
                let (digits, radix) = if let Some(n) =
                    text.strip_prefix("0x").or_else(|| text.strip_prefix("0X"))
                {
                    (n, 16)
                } else if let Some(n) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O"))
                {
                    (n, 8)
                } else if let Some(n) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B"))
                {
                    (n, 2)
                } else {
                    (text, 10)
                };
                let compact = digits.replace('_', "");
                if !compact.is_empty() && compact.chars().all(|c| c.is_digit(radix)) {
                    ensure!(
                        radix != 2
                            && !digits.contains('_')
                            && !(radix == 10 && digits.len() > 1 && digits.starts_with('0')),
                        "non-canonical integer spelling; use decimal without leading zeros or separators, or quote it as a string"
                    );
                    let value = u64::from_str_radix(digits, radix)
                        .map_err(|_| anyhow::anyhow!("integer outside i64/u64 range"))?;
                    ensure!(
                        !scalar.starts_with('-') || value <= (1u64 << 63),
                        "integer outside i64 range"
                    );
                }
            }
            _ => (),
        }
    }
    let mut options = serde_saphyr::Options::default();
    options.duplicate_keys = serde_saphyr::DuplicateKeyPolicy::Error;
    options.merge_keys = serde_saphyr::MergeKeyPolicy::Error;
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    options.reject_non_finite_typeless_float = true;
    Ok(serde_saphyr::from_str_with_options::<StrictValue>(yaml, options)?.0)
}

struct StrictValue(Value);
impl<'de> serde::Deserialize<'de> for StrictValue {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StrictValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a finite JSON-compatible metadata value")
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_bool<E: serde::de::Error>(
                self,
                v: bool,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> std::result::Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| StrictValue(n.into()))
                    .ok_or_else(|| E::custom("non-finite number"))
            }
            fn visit_str<E: serde::de::Error>(
                self,
                v: &str,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_string<E: serde::de::Error>(
                self,
                v: String,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut v = Vec::new();
                while let Some(StrictValue(item)) = a.next_element()? {
                    v.push(item);
                }
                Ok(StrictValue(Value::Array(v)))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                use serde::de::Error;
                let mut v = Mapping::new();
                while let Some(StrictValue(key)) = a.next_key()? {
                    let Value::String(key) = key else {
                        return Err(A::Error::custom("metadata keys must be strings"));
                    };
                    if key == "<<" || v.contains_key(&key) {
                        return Err(A::Error::custom("merge or duplicate key"));
                    }
                    let StrictValue(value) = a.next_value()?;
                    v.insert(key, value);
                }
                Ok(StrictValue(Value::Object(v)))
            }
        }
        d.deserialize_any(Visitor)
    }
}

#[derive(Default)]
pub struct Mutation {
    pub title: Option<String>,
    pub tags: Option<String>,
    pub set: Vec<String>,
    pub set_json: Vec<String>,
    pub unset: Vec<String>,
}
impl Mutation {
    pub fn apply(&self, map: &mut Mapping) -> Result<()> {
        let mut operations = Vec::new();
        if let Some(t) = &self.title {
            operations.push(("title".to_owned(), Some(t.clone().into())));
        }
        if let Some(t) = &self.tags {
            operations.push((
                "tags".to_owned(),
                Some(serde_json::to_value(
                    t.split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>(),
                )?),
            ));
        }
        for (typed, values) in [(false, &self.set), (true, &self.set_json)] {
            for assignment in values {
                let (key, value) = assignment
                    .split_once('=')
                    .ok_or_else(|| anyhow::anyhow!("assignment requires KEY=VALUE"))?;
                operations.push((
                    key.into(),
                    Some(if typed {
                        {
                            let json = serde_json::from_str::<StrictValue>(value)?.0;
                            // Use our lexical numeric bounds for JSON too; JSON
                            // alone would round oversized integer literals.
                            parse(value)?;
                            json
                        }
                    } else {
                        value.into()
                    }),
                ));
            }
        }
        operations.extend(self.unset.iter().map(|k| (k.clone(), None)));
        let mut seen = std::collections::BTreeSet::new();
        for (key, _) in &operations {
            ensure!(
                !key.trim().is_empty() && !key.chars().any(char::is_control) && key != "<<",
                "invalid metadata key"
            );
            // Removing an invalid reserved value is the non-editor repair path;
            // valid dates stay managed automatically.
            let repair = self.unset.contains(key)
                && map
                    .get(key)
                    .is_some_and(|v| crate::bit::timestamp(v).is_none());
            ensure!(
                !["created", "updated"].contains(&key.as_str()) || repair,
                "{key} is reserved and managed automatically"
            );
            ensure!(
                seen.insert(key),
                "conflicting metadata operations for {key}"
            );
        }
        for (key, value) in operations {
            if let Some(value) = value {
                map.insert(key, value);
            } else {
                map.remove(&key);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frontmatter_corpus() {
        for category in ["accepted", "rejected"] {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/frontmatter")
                .join(category);
            for entry in std::fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                let raw = std::fs::read_to_string(&path).unwrap();
                let parsed = crate::bit::parse(&raw);
                if category == "accepted" {
                    let (map, body) =
                        parsed.unwrap_or_else(|e| panic!("{}: {e:#}", path.display()));
                    let rendered = crate::lifecycle::render(&map, body).unwrap();
                    let (again, again_body) = crate::bit::parse(&rendered).unwrap();
                    assert_eq!(map, again);
                    assert_eq!(body.as_bytes(), again_body.as_bytes());
                } else {
                    assert!(parsed.is_err(), "{} was accepted", path.display());
                }
            }
        }
        for yaml in [
            "x: !!str 1",
            "x: {true: a}",
            "x: 0x10000000000000000",
            "x: -9223372036854775809",
            "x: {a: 1, a: 2}",
        ] {
            assert!(parse(yaml).is_err(), "{yaml}");
        }
        assert_eq!(parse("x: yes").unwrap()["x"], "yes");
    }
}
