#![allow(unused)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod apply;
pub mod case_model;
pub mod history;
pub mod pattern;
pub mod preview;
pub mod rename;
pub mod scanner;
pub mod undo;

pub use apply::{apply_plan, ApplyOptions};
pub use case_model::{
    detect_style, generate_variant_map, parse_to_tokens, to_style, Style, Token, TokenModel,
};
pub use history::{
    create_history_entry, format_history, get_status, History, HistoryEntry, StatusInfo,
};
pub use pattern::{build_pattern, find_matches, is_boundary, Match, MatchPattern};
pub use preview::{render_plan, write_preview, PreviewFormat};
pub use rename::{
    detect_case_insensitive_fs, plan_renames_with_conflicts, ConflictKind, RenameConflict,
    RenamePlan,
};
pub use scanner::{
    scan_repository, write_plan, MatchHunk, Plan, PlanOptions, Rename, RenameKind, Stats,
};
pub use undo::{redo_refactoring, undo_refactoring};

use ignore::WalkBuilder;
use std::path::Path;

/// Configure a WalkBuilder based on the unrestricted level in PlanOptions.
/// 
/// This matches ripgrep's behavior:
/// - Level 0 (default): Respect all ignore files, skip hidden files
/// - Level 1 (-u): Don't respect .gitignore, but respect other ignore files, skip hidden  
/// - Level 2 (-uu): Don't respect any ignore files, show hidden files
/// - Level 3 (-uuu): Same as level 2, plus treat binary files as text (handled by caller)
pub fn configure_walker(root: &Path, options: &scanner::PlanOptions) -> WalkBuilder {
    let mut builder = WalkBuilder::new(root);
    
    // Map unrestricted level to ignore settings
    // Note: respect_gitignore is kept for backward compatibility
    let level = if !options.respect_gitignore && options.unrestricted_level == 0 {
        1  // Legacy flag takes precedence if set
    } else {
        options.unrestricted_level
    };
    
    match level {
        0 => {
            // Default: respect all ignore files, skip hidden
            builder
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .ignore(true)
                .parents(true)
                .hidden(true)  // true = skip hidden files
                .add_custom_ignore_filename(".rgignore");
        }
        1 => {
            // -u: Don't respect .gitignore, but respect others, skip hidden
            builder
                .git_ignore(false)  // Don't respect .gitignore
                .git_global(true)   // Still respect global gitignore
                .git_exclude(true)  // Still respect .git/info/exclude
                .ignore(true)       // Still respect .ignore/.rgignore
                .parents(true)      // Still check parent dirs
                .hidden(true)       // Still skip hidden files
                .add_custom_ignore_filename(".rgignore");
        }
        2 | 3 => {
            // -uu/-uuu: Don't respect any ignore files, show hidden
            // Level 3 also treats binary as text, but that's handled by scanner
            builder
                .git_ignore(false)
                .git_global(false)
                .git_exclude(false)
                .ignore(false)
                .parents(false)
                .hidden(false);  // false = show hidden files
        }
        _ => {
            // Treat any higher level as maximum unrestricted
            builder
                .git_ignore(false)
                .git_global(false)
                .git_exclude(false)
                .ignore(false)
                .parents(false)
                .hidden(false);
        }
    }
    
    builder
}

