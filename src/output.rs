use anyhow::Result;
use serde::Serialize;
use std::io::Write;

/// Use fallible writes (including flush) so main can handle a closed pipe quietly.
pub fn write(bytes: &[u8]) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.flush()?;
    Ok(())
}

pub fn line(value: impl std::fmt::Display) -> Result<()> {
    write(format!("{value}\n").as_bytes())
}

pub fn emit(value: &impl Serialize, json: bool, human: impl AsRef<str>) -> Result<()> {
    if json {
        line(serde_json::to_string(value)?)
    } else if !human.as_ref().is_empty() {
        line(human.as_ref())
    } else {
        Ok(())
    }
}

pub fn check_listing(json: bool, long: bool, paths: bool, null: bool) -> Result<()> {
    crate::usage_check(
        !(json && (long || paths || null)),
        "--json cannot be combined with --long, --paths or --null",
    )?;
    crate::usage_check(
        !(long && (paths || null)),
        "--long cannot be combined with --paths or --null",
    )
}

pub fn listing(
    bits: &[crate::bit::Bit],
    json: bool,
    long: bool,
    paths: bool,
    null: bool,
) -> Result<()> {
    if json {
        return emit(&bits, true, "");
    }
    let mut bytes = Vec::new();
    for bit in bits {
        if paths {
            // Paths can contain non-UTF-8 bytes on Unix. Do not lossy-convert
            // filenames intended for downstream filesystem tools.
            bytes.extend_from_slice(bit.path.as_os_str().as_encoded_bytes());
        } else {
            bytes.extend_from_slice(bit.id.as_bytes());
            if long {
                bytes.push(b'\t');
                bytes.extend_from_slice(bit.title.as_deref().unwrap_or("").as_bytes());
            }
        }
        bytes.push(if null { 0 } else { b'\n' });
    }
    write(&bytes)
}
