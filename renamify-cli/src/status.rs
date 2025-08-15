use anyhow::{Context, Result};
use renamify_core::get_status;
use std::path::PathBuf;

pub fn handle_status() -> Result<()> {
    let renamify_dir = PathBuf::from(".renamify");
    let status = get_status(&renamify_dir).context("Failed to get status")?;

    print!("{}", status.format());
    Ok(())
}
