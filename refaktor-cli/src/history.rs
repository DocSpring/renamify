use anyhow::{Context, Result};
use refaktor_core::{format_history, History};
use std::path::PathBuf;

pub fn handle_history(limit: Option<usize>) -> Result<()> {
    let refaktor_dir = PathBuf::from(".refaktor");
    let history = History::load(&refaktor_dir).context("Failed to load history")?;

    let entries = history.list_entries(limit);
    let formatted = format_history(&entries, false)?;

    println!("{}", formatted);
    Ok(())
}