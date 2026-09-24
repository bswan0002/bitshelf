use anyhow::{Result, ensure};
use std::io::IsTerminal;
pub fn require(json: bool) -> Result<()> {
    ensure!(
        !json && std::io::stdin().is_terminal() && std::io::stdout().is_terminal(),
        "interactive mode requires a terminal and cannot use --json; supply explicit arguments instead"
    );
    Ok(())
}
pub fn input(label: &str) -> Result<String> {
    demand::Input::new(label)
        .run()
        .map_err(|e| anyhow::anyhow!("prompt cancelled or failed: {e}"))
}
pub fn select(label: &str, values: &[String]) -> Result<String> {
    ensure!(
        !values.is_empty(),
        "no shelves available; use bs shelf add NAME"
    );
    let mut prompt = demand::Select::new(label).filterable(true);
    for v in values {
        prompt = prompt.option(demand::DemandOption::new(v.clone()));
    }
    prompt
        .run()
        .map_err(|e| anyhow::anyhow!("prompt cancelled or failed: {e}"))
}
pub fn pick(values: &[String]) -> Result<Vec<String>> {
    ensure!(!values.is_empty(), "no bits to open");
    let mut prompt = demand::MultiSelect::new("Open bits")
        .filterable(true)
        .min(1);
    for v in values {
        prompt = prompt.option(demand::DemandOption::new(v.clone()));
    }
    prompt
        .run()
        .map_err(|e| anyhow::anyhow!("prompt cancelled or failed: {e}"))
}
