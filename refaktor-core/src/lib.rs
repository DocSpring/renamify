#![allow(unused)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod apply;
pub mod case_model;
pub mod pattern;
pub mod preview;
pub mod rename;
pub mod scanner;

pub use apply::{apply_plan, ApplyOptions};
pub use case_model::{
    detect_style, generate_variant_map, parse_to_tokens, to_style, Style, Token, TokenModel,
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

