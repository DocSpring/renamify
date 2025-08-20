use anyhow::Result;
use renamify_core::{plan_operation, OutputFormatter, Style};
use std::path::PathBuf;

use crate::cli::{types::StyleArg, OutputFormat};
use renamify_core::Preview;

#[allow(clippy::too_many_arguments)]
pub fn handle_plan(
    search: &str,
    replace: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    respect_gitignore: bool,
    unrestricted: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: Vec<StyleArg>,
    include_styles: Vec<StyleArg>,
    only_styles: Vec<StyleArg>,
    exclude_match: Vec<String>,
    exclude_matching_lines: Option<String>,
    preview: Option<Preview>,
    fixed_table_width: bool,
    plan_out: PathBuf,
    dry_run: bool,
    use_color: bool,
    no_acronyms: bool,
    include_acronyms: Vec<String>,
    exclude_acronyms: Vec<String>,
    only_acronyms: Vec<String>,
    output: OutputFormat,
    quiet: bool,
) -> Result<()> {
    // Error if both preview and JSON output are specified
    if preview.is_some() && preview != Some(Preview::None) && output == OutputFormat::Json {
        return Err(anyhow::anyhow!(
            "Cannot use both --preview and --output json. Use --output json for machine-readable output."
        ));
    }
    // Validate that --fixed-table-width is only used with table preview
    if fixed_table_width && preview.is_some() && preview != Some(Preview::Table) {
        return Err(anyhow::anyhow!(
            "--fixed-table-width can only be used with --preview table"
        ));
    }

    // Convert CLI style args to core Style enum
    let exclude_styles: Vec<Style> = exclude_styles.into_iter().map(Into::into).collect();
    let include_styles: Vec<Style> = include_styles.into_iter().map(Into::into).collect();
    let only_styles: Vec<Style> = only_styles.into_iter().map(Into::into).collect();

    // Handle quiet mode - overrides preview to none unless output is json
    let effective_preview = if quiet && output != OutputFormat::Json {
        None
    } else {
        preview
    };

    // For JSON output, don't generate preview
    let preview_format = if output == OutputFormat::Json {
        None
    } else {
        effective_preview.map(|p| match p {
            Preview::Table => "table".to_string(),
            Preview::Diff => "diff".to_string(),
            Preview::Matches => "matches".to_string(),
            Preview::Summary => "summary".to_string(),
            Preview::None => "none".to_string(),
        })
    };

    // Call the core operation
    let (result, preview_content) = plan_operation(
        search,
        replace,
        paths,
        include,
        exclude,
        respect_gitignore,
        unrestricted,
        rename_files,
        rename_dirs,
        &exclude_styles,
        &include_styles,
        &only_styles,
        exclude_match,
        exclude_matching_lines,
        Some(plan_out),
        preview_format.as_ref(),
        dry_run,
        fixed_table_width,
        use_color,
        no_acronyms,
        include_acronyms,
        exclude_acronyms,
        only_acronyms,
        None, // working_dir
    )?;

    // Handle output based on format
    match output {
        OutputFormat::Json => {
            print!("{}", result.format_json());
        },
        OutputFormat::Summary => {
            if !quiet {
                // Print preview content if available
                if let Some(preview) = preview_content {
                    println!("{}", preview);
                }
                // Print summary
                print!("{}", result.format_summary());
            }
        },
    }

    Ok(())
}
