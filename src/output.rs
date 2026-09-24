use anyhow::Result;
use serde::Serialize;
pub fn emit(value: &impl Serialize, json: bool, human: impl AsRef<str>) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(value)?);
    } else if !human.as_ref().is_empty() {
        println!("{}", human.as_ref());
    }
    Ok(())
}
