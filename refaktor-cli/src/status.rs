use anyhow::{Context, Result};
use refaktor_core::get_status;
use std::path::PathBuf;

pub fn handle_status() -> Result<()> {
    let refaktor_dir = PathBuf::from(".refaktor");
    let status = get_status(&refaktor_dir).context("Failed to get status")?;

    print!("{}", status.format());
    Ok(())
}
