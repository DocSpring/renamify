use anyhow::Result;
use renamify_core::{redo_operation, OutputFormatter};

use crate::OutputFormat;

pub fn handle_redo(id: &str, output: OutputFormat, quiet: bool) -> Result<()> {
    let result = redo_operation(id, None)?;

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
