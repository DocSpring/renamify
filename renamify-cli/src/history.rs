use anyhow::{Context, Result};
use renamify_core::{format_history, History};
use std::path::PathBuf;

pub fn handle_history(limit: Option<usize>) -> Result<()> {
    let renamify_dir = PathBuf::from(".renamify");
    let history = History::load(&renamify_dir).context("Failed to load history")?;

    let entries = history.list_entries(limit);
    let formatted = format_history(&entries, false)?;

    println!("{}", formatted);
    Ok(())
}
