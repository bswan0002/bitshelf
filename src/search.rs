//! Predictable, index-free search. Normalize each field once per candidate.
use crate::{bit::Bit, usage_check};
use anyhow::Result;
use serde::Serialize;

struct Term {
    text: String,
    normalized: String,
    phrase: bool,
    excluded: bool,
}

pub struct Query {
    terms: Vec<Term>,
    any: bool,
}

#[derive(Serialize)]
pub struct Match {
    pub fields: Vec<&'static str>,
    pub score: usize,
}

fn normalize(text: &str) -> String {
    text.chars()
        .map(|c| {
            if matches!(c, '-' | '_' | '.' | '/' | ':') {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl Query {
    pub fn parse(input: &str, any: bool) -> Result<Self> {
        let mut chars = input.chars().peekable();
        let mut terms = Vec::new();
        while chars.peek().is_some() {
            if chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
                continue;
            }
            let excluded = chars.peek() == Some(&'!');
            if excluded {
                chars.next();
            }
            let phrase = chars.peek() == Some(&'"');
            if phrase {
                chars.next();
            }
            let mut text = String::new();
            let mut closed = !phrase;
            while let Some(c) = chars.next() {
                if c == '\\' {
                    let next = chars.next();
                    usage_check(
                        next.is_some(),
                        "search query ends with an incomplete escape",
                    )?;
                    text.push(next.unwrap());
                } else if c == '"' {
                    usage_check(phrase, "quotes must start a search term")?;
                    closed = true;
                    break;
                } else if !phrase && c.is_whitespace() {
                    break;
                } else {
                    text.push(c);
                }
            }
            usage_check(closed, "unterminated quoted search phrase")?;
            if phrase {
                usage_check(
                    chars.peek().is_none_or(|c| c.is_whitespace()),
                    "quoted search phrases must be separated by whitespace",
                )?;
            }
            usage_check(!text.trim().is_empty(), "search terms must not be empty")?;
            let text = text.to_lowercase();
            if !terms
                .iter()
                .any(|t: &Term| t.text == text && t.phrase == phrase && t.excluded == excluded)
            {
                terms.push(Term {
                    normalized: if phrase {
                        String::new()
                    } else {
                        normalize(&text)
                    },
                    text,
                    phrase,
                    excluded,
                });
            }
        }
        usage_check(!terms.is_empty(), "search query must not be empty")?;
        Ok(Self { terms, any })
    }

    pub fn matches(&self, bit: &Bit) -> Option<Match> {
        // Keep tags separate: phrases must not span unrelated metadata values.
        let mut fields = vec![
            ("id", 8, bit.id.to_lowercase()),
            (
                "title",
                8,
                bit.title.as_deref().unwrap_or("").to_lowercase(),
            ),
            ("body", 1, bit.body.to_lowercase()),
        ];
        fields.extend(bit.tags.iter().map(|tag| ("tags", 4, tag.to_lowercase())));
        let normalized: Vec<_> = fields
            .iter()
            .map(|(name, _, value)| {
                if matches!(*name, "id" | "tags") {
                    normalize(value)
                } else {
                    String::new()
                }
            })
            .collect();
        let mut matched_fields = Vec::new();
        let mut score = 0;
        let mut positives = 0;
        let mut matched = 0;
        for term in &self.terms {
            let mut best = 0;
            for (index, (name, weight, value)) in fields.iter().enumerate() {
                let hit = value.contains(&term.text)
                    || (!term.phrase
                        && matches!(*name, "id" | "tags")
                        && !term.normalized.is_empty()
                        && normalized[index].contains(&term.normalized));
                if hit {
                    if term.excluded {
                        return None;
                    }
                    best = best.max(*weight);
                    if !matched_fields.contains(name) {
                        matched_fields.push(*name);
                    }
                }
            }
            if !term.excluded {
                positives += 1;
                if best > 0 {
                    matched += 1;
                }
                score += best;
            }
        }
        if positives > 0 && (matched == 0 || (!self.any && matched != positives)) {
            return None;
        }
        // A stable schema order, independent of query term order.
        matched_fields.sort_by_key(|name| match *name {
            "id" => 0,
            "title" => 1,
            "tags" => 2,
            _ => 3,
        });
        Some(Match {
            fields: matched_fields,
            score,
        })
    }
}
