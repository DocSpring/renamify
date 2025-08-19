use anyhow::Result;
use renamify_core::{status_operation, OutputFormatter};

use crate::OutputFormat;

pub fn handle_status(output: OutputFormat, quiet: bool) -> Result<()> {
    let result = status_operation(None)?;

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
