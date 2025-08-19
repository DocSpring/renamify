use anyhow::Result;
use renamify_core::{undo_operation, OutputFormatter};

use crate::OutputFormat;

pub fn handle_undo(id: &str, output: OutputFormat, quiet: bool) -> Result<()> {
    let result = undo_operation(id, None)?;

    // Handle output based on format
    match output {
        OutputFormat::Json => {
            print!("{}", result.format_json());
        },
        OutputFormat::Summary => {
            if !quiet {
                print!("{}", result.format_summary());
            }
        },
    }

    Ok(())
}
