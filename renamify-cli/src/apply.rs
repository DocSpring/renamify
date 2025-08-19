use anyhow::Result;
use renamify_core::{apply_operation, OutputFormatter};

use crate::OutputFormat;

pub fn handle_apply(
    plan_id: Option<String>,
    commit: bool,
    force: bool,
    output: OutputFormat,
    quiet: bool,
) -> Result<()> {
    let result = apply_operation(None, plan_id, commit, force, None)?;

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
