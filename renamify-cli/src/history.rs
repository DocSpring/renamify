use anyhow::Result;
use renamify_core::{history_operation, OutputFormatter};

use crate::OutputFormat;

pub fn handle_history(limit: Option<usize>, output: OutputFormat, quiet: bool) -> Result<()> {
    let result = history_operation(limit, None)?;

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
