use anyhow::Result;
use renamify_core::{plan_operation_with_dry_run, Style};
use std::path::PathBuf;

use crate::StyleArg;
use renamify_core::Preview;

#[allow(clippy::too_many_arguments)]
pub fn handle_plan(
    old: &str,
    new: &str,
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
    preview: Option<Preview>,
    fixed_table_width: bool,
    plan_out: PathBuf,
    dry_run: bool,
    use_color: bool,
    no_acronyms: bool,
    include_acronyms: Vec<String>,
    exclude_acronyms: Vec<String>,
    only_acronyms: Vec<String>,
) -> Result<()> {
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

    // Convert preview arg to string format
    let preview_format = preview.map(|p| match p {
        Preview::Table => "table".to_string(),
        Preview::Diff => "diff".to_string(),
        Preview::Json => "json".to_string(),
        Preview::Summary => "summary".to_string(),
        Preview::None => "none".to_string(),
    });

    // Call the core operation
    let result = plan_operation_with_dry_run(
        old,
        new,
        paths,
        include,
        exclude,
        respect_gitignore,
        unrestricted,
        rename_files,
        rename_dirs,
        exclude_styles,
        include_styles,
        only_styles,
        exclude_match,
        Some(plan_out),
        preview_format,
        dry_run,
        fixed_table_width,
        use_color,
        no_acronyms,
        include_acronyms,
        exclude_acronyms,
        only_acronyms,
    )?;

    println!("{}", result);
    Ok(())
}
