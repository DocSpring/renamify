use anyhow::Result;
use renamify_core::{rename_operation, Style};
use std::path::PathBuf;

use crate::{PreviewArg, StyleArg};

#[allow(clippy::too_many_arguments)]
pub fn handle_rename(
    old: &str,
    new: &str,
    paths: Vec<PathBuf>,
    include: Vec<String>,
    exclude: Vec<String>,
    unrestricted: u8,
    rename_files: bool,
    rename_dirs: bool,
    exclude_styles: Vec<StyleArg>,
    include_styles: Vec<StyleArg>,
    only_styles: Vec<StyleArg>,
    exclude_match: Vec<String>,
    exclude_matching_lines: Option<String>,
    preview: Option<PreviewArg>,
    commit: bool,
    large: bool,
    force_with_conflicts: bool,
    _confirm_collisions: bool, // TODO: implement collision detection
    rename_root: bool,
    no_rename_root: bool,
    dry_run: bool,
    no_acronyms: bool,
    include_acronyms: Vec<String>,
    exclude_acronyms: Vec<String>,
    only_acronyms: Vec<String>,
    auto_approve: bool,
    use_color: bool,
) -> Result<()> {
    // Convert CLI style args to core Style enum
    let exclude_styles: Vec<Style> = exclude_styles.into_iter().map(Into::into).collect();
    let include_styles: Vec<Style> = include_styles.into_iter().map(Into::into).collect();
    let only_styles: Vec<Style> = only_styles.into_iter().map(Into::into).collect();

    // Convert preview arg to string format
    let preview_format = preview.map(|p| match p {
        PreviewArg::Table => "table".to_string(),
        PreviewArg::Diff => "diff".to_string(),
        PreviewArg::Json => "json".to_string(),
        PreviewArg::Summary => "summary".to_string(),
        PreviewArg::None => "none".to_string(),
    });

    // Call the core operation
    let result = rename_operation(
        old,
        new,
        paths,
        &include,
        &exclude,
        unrestricted,
        rename_files,
        rename_dirs,
        &exclude_styles,
        &include_styles,
        &only_styles,
        &exclude_match,
        exclude_matching_lines.as_ref(),
        preview_format.as_ref(),
        commit,
        large,
        force_with_conflicts,
        rename_root,
        no_rename_root,
        dry_run,
        no_acronyms,
        &include_acronyms,
        &exclude_acronyms,
        &only_acronyms,
        auto_approve,
        use_color,
    )?;

    // Print the result
    println!("{}", result);
    Ok(())
}
