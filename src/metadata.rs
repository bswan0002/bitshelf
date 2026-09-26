//! One supported value model for JSON, semantic comparison and mutations.
use anyhow::{Result, ensure};
pub type Mapping = serde_json::Map<String, serde_json::Value>;
pub use serde_json::Value;

pub fn from_yaml(value: serde_yaml::Value) -> Result<Value> {
    use serde_yaml::Value as Y;
    Ok(match value {
        Y::Null => Value::Null,
        Y::Bool(b) => b.into(),
        Y::String(s) => s.into(),
        Y::Number(n) => {
            if let Some(n) = n.as_i64() {
                n.into()
            } else if let Some(n) = n.as_u64() {
                n.into()
            } else {
                serde_json::Number::from_f64(n.as_f64().unwrap())
                    .ok_or_else(|| anyhow::anyhow!("non-finite metadata number"))?
                    .into()
            }
        }
        Y::Sequence(a) => Value::Array(a.into_iter().map(from_yaml).collect::<Result<_>>()?),
        Y::Mapping(m) => {
            let mut result = Mapping::new();
            for (key, value) in m {
                let key = key
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("metadata keys must be strings"))?;
                ensure!(key != "<<", "YAML merge keys are unsupported");
                result.insert(key.into(), from_yaml(value)?);
            }
            Value::Object(result)
        }
        Y::Tagged(_) => anyhow::bail!("explicit YAML tags are unsupported"),
    })
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
                        serde_json::from_str(value)?
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
            ensure!(
                !["created", "updated"].contains(&key.as_str()),
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
