use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use super::types::{OutputFormat, PreviewArg, SearchPreviewArg, StyleArg};

/// Smart search & replace for code and files with case-aware transformations
#[derive(Parser, Debug)]
#[command(name = "renamify")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,

    /// Reduce the level of "smart" filtering. Can be repeated up to 3 times.
    /// -u: Don't respect .gitignore files
    /// -uu: Don't respect any ignore files (.gitignore, .ignore, .rgignore, .rnignore), include hidden files
    /// -uuu: Same as -uu, plus treat binary files as text
    #[arg(short = 'u', long = "unrestricted", global = true, action = clap::ArgAction::Count, verbatim_doc_comment)]
    pub unrestricted: u8,

    /// Run as if started in <path> instead of the current working directory
    #[arg(short = 'C', global = true, value_name = "PATH")]
    pub directory: Option<PathBuf>,

    /// Automatically initialize .renamify ignore (repo|local|global)
    #[arg(long, global = true, value_name = "MODE")]
    pub auto_init: Option<String>,

    /// Disable automatic initialization prompt
    #[arg(long, global = true, conflicts_with = "auto_init")]
    pub no_auto_init: bool,

    /// Assume yes for all prompts
    #[arg(short = 'y', long = "yes", global = true, env = "RENAMIFY_YES")]
    pub yes: bool,
}

/// Common style arguments shared across multiple commands
#[derive(Args, Debug, Clone)]
pub struct StyleArgs {
    /// Case styles to exclude from the default set (snake, kebab, camel, pascal, screaming-snake, train, screaming-train)
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        conflicts_with = "only_styles"
    )]
    pub exclude_styles: Vec<StyleArg>,

    /// Additional case styles to include (title, dot, lower, upper)
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        conflicts_with = "only_styles"
    )]
    pub include_styles: Vec<StyleArg>,

    /// Use only these case styles (overrides defaults)
    #[arg(long, value_enum, value_delimiter = ',', conflicts_with_all = ["exclude_styles", "include_styles"])]
    pub only_styles: Vec<StyleArg>,

    /// Ignore mixed-case/ambiguous identifiers that don't match standard patterns
    #[arg(long)]
    pub ignore_ambiguous: bool,
}

/// Common path filtering arguments
#[derive(Args, Debug, Clone)]
pub struct FilterArgs {
    /// Include glob patterns
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,

    /// Exclude glob patterns
    #[arg(long, value_delimiter = ',')]
    pub exclude: Vec<String>,

    /// Respect ignore files (.gitignore, .ignore, .rgignore, .rnignore)
    #[arg(long, default_value_t = true)]
    pub respect_gitignore: bool,
}

/// Common file renaming arguments
#[derive(Args, Debug, Clone)]
pub struct RenameFileArgs {
    /// Don't rename matching files
    #[arg(long = "no-rename-files")]
    pub no_rename_files: bool,

    /// Don't rename matching directories
    #[arg(long = "no-rename-dirs")]
    pub no_rename_dirs: bool,

    /// Don't rename files or directories (equivalent to --no-rename-files --no-rename-dirs)
    #[arg(long = "no-rename-paths")]
    pub no_rename_paths: bool,
}

/// Common acronym arguments shared across commands
#[derive(Args, Debug, Clone)]
pub struct AcronymArgs {
    /// Disable acronym detection (treat CLI, API, etc. as regular words)
    #[arg(long)]
    pub no_acronyms: bool,

    /// Additional acronyms to recognize (comma-separated, e.g., "AWS,GCP,K8S")
    #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
    pub include_acronyms: Vec<String>,

    /// Default acronyms to exclude (comma-separated, e.g., "ID,UI")
    #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
    pub exclude_acronyms: Vec<String>,

    /// Use only these acronyms (replaces default list)
    #[arg(long, value_delimiter = ',', conflicts_with_all = ["include_acronyms", "exclude_acronyms"])]
    pub only_acronyms: Vec<String>,
}

/// Atomic identifier arguments
#[derive(Args, Debug, Clone)]
pub struct AtomicArgs {
    /// Treat both terms as atomic (single words). E.g. DocSpring becomes docspring in snake_case, not doc_spring
    #[arg(long, conflicts_with_all = ["atomic_search", "atomic_replace"])]
    pub atomic: bool,

    /// Treat search term as atomic (DocSpring → docspring, not doc_spring)
    #[arg(long)]
    pub atomic_search: bool,

    /// Treat replace term as atomic (DocSpring → docspring, not doc_spring)
    #[arg(long)]
    pub atomic_replace: bool,

    /// Override config: allow word boundary detection
    #[arg(long, conflicts_with = "atomic")]
    pub no_atomic: bool,

    /// Override config for search: allow word boundaries
    #[arg(long, conflicts_with = "atomic_search")]
    pub no_atomic_search: bool,

    /// Override config for replace: allow word boundaries
    #[arg(long, conflicts_with = "atomic_replace")]
    pub no_atomic_replace: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize renamify in the current repository
    Init {
        /// Add to .git/info/exclude instead of .gitignore
        #[arg(long, conflicts_with = "global")]
        local: bool,

        /// Add to global git excludes file
        #[arg(long, conflicts_with = "local")]
        global: bool,

        /// Check if .renamify is ignored (exit 0 if yes, 1 if no)
        #[arg(long, conflicts_with_all = ["local", "global", "configure_global"])]
        check: bool,

        /// Configure global excludes file if it doesn't exist
        #[arg(long, requires = "global")]
        configure_global: bool,
    },

    /// Search for identifiers without creating a plan
    Search {
        /// Identifier to search for
        term: String,

        /// Paths to search (files or directories). Defaults to current directory
        #[arg(help = "Search paths (files or directories)")]
        paths: Vec<PathBuf>,

        /// Include glob patterns
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Exclude glob patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Rename files and directories (default: true)
        #[arg(long, default_value_t = true)]
        rename_files: bool,

        /// Rename directories (default: true)
        #[arg(long, default_value_t = true)]
        rename_dirs: bool,

        #[command(flatten)]
        styles: StyleArgs,

        /// Exclude matches on lines matching this regex pattern
        #[arg(long)]
        exclude_matching_lines: Option<String>,

        /// Preview output format (defaults from config if not specified)
        #[arg(long, value_enum)]
        preview: Option<SearchPreviewArg>,

        /// Use fixed column widths for table output (useful in CI environments or other non-TTY use cases)
        #[arg(long)]
        fixed_table_width: bool,

        #[command(flatten)]
        acronyms: AcronymArgs,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Plan and apply a renaming in one step (with confirmation)
    Rename {
        /// Old identifier to replace
        search: String,

        /// New identifier to replace with
        replace: String,

        /// Paths to search (files or directories). Defaults to current directory
        #[arg(help = "Search paths (files or directories)")]
        paths: Vec<PathBuf>,

        #[command(flatten)]
        filter: FilterArgs,

        #[command(flatten)]
        rename_files: RenameFileArgs,

        #[command(flatten)]
        styles: StyleArgs,

        /// Specific matches to exclude (e.g., compound words to ignore)
        #[arg(long, value_delimiter = ',')]
        exclude_match: Vec<String>,

        /// Exclude matches on lines matching this regex pattern
        #[arg(long)]
        exclude_matching_lines: Option<String>,

        /// Show preview before confirmation prompt
        #[arg(long, value_enum)]
        preview: Option<PreviewArg>,

        /// Commit changes to git after applying
        #[arg(long)]
        commit: bool,

        /// Acknowledge large changes (>500 files or >100 renames)
        #[arg(long)]
        large: bool,

        /// Force apply even with conflicts
        #[arg(long)]
        force_with_conflicts: bool,

        /// Confirm case-insensitive or collision renames
        #[arg(long)]
        confirm_collisions: bool,

        /// Actually rename the root project directory (requires confirmation)
        #[arg(long)]
        rename_root: bool,

        /// Never rename the root project directory
        #[arg(long, conflicts_with = "rename_root")]
        no_rename_root: bool,

        /// Show preview only, don't apply changes
        #[arg(long)]
        dry_run: bool,

        #[command(flatten)]
        acronyms: AcronymArgs,

        #[command(flatten)]
        atomic: AtomicArgs,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Simple regex or literal string replacement
    Replace {
        /// Search pattern (regex by default, literal with --no-regex)
        pattern: String,

        /// Replacement string (supports $1, $2 capture groups in regex mode)
        replacement: String,

        /// Paths to search (files or directories). Defaults to current directory
        #[arg(help = "Search paths (files or directories)")]
        paths: Vec<PathBuf>,

        /// Treat pattern as literal string instead of regex
        #[arg(long = "no-regex")]
        no_regex: bool,

        #[command(flatten)]
        filter: FilterArgs,

        #[command(flatten)]
        rename_files: RenameFileArgs,

        /// Exclude matches on lines matching this regex pattern
        #[arg(long)]
        exclude_matching_lines: Option<String>,

        /// Show preview before confirmation prompt
        #[arg(long, value_enum)]
        preview: Option<PreviewArg>,

        /// Commit changes to git after applying
        #[arg(long)]
        commit: bool,

        /// Acknowledge large changes (>500 files or >100 renames)
        #[arg(long)]
        large: bool,

        /// Force apply even with conflicts
        #[arg(long)]
        force_with_conflicts: bool,

        /// Show preview only, don't apply changes
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt and apply immediately
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Generate a renaming plan
    Plan {
        /// Old identifier to replace
        search: String,

        /// New identifier to replace with
        replace: String,

        /// Paths to search (files or directories). Defaults to current directory
        #[arg(help = "Search paths (files or directories)")]
        paths: Vec<PathBuf>,

        #[command(flatten)]
        filter: FilterArgs,

        #[command(flatten)]
        rename_files: RenameFileArgs,

        #[command(flatten)]
        styles: StyleArgs,

        /// Specific matches to exclude (e.g., compound words to ignore)
        #[arg(long, value_delimiter = ',')]
        exclude_match: Vec<String>,

        /// Exclude matches on lines matching this regex pattern
        #[arg(long)]
        exclude_matching_lines: Option<String>,

        /// Preview output format (defaults from config if not specified)
        #[arg(long, value_enum)]
        preview: Option<PreviewArg>,

        /// Use fixed column widths for table output (useful in CI environments or other non-TTY use cases)
        #[arg(long)]
        fixed_table_width: bool,

        /// Output path for the plan
        #[arg(long, default_value = ".renamify/plan.json")]
        plan_out: PathBuf,

        /// Only show preview, don't write plan (dry-run)
        #[arg(long)]
        dry_run: bool,

        #[command(flatten)]
        acronyms: AcronymArgs,

        #[command(flatten)]
        atomic: AtomicArgs,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Apply a renaming plan
    Apply {
        /// Plan ID or path to apply (optional - defaults to .renamify/plan.json)
        id: Option<String>,

        /// Apply changes atomically
        #[arg(long, default_value_t = true)]
        atomic: bool,

        /// Commit changes to git
        #[arg(long)]
        commit: bool,

        /// Force apply even with conflicts
        #[arg(long)]
        force_with_conflicts: bool,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Undo a previous renaming
    Undo {
        /// History ID to undo (use 'latest' for the most recent non-revert entry)
        id: String,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Redo a previously undone renaming
    Redo {
        /// History ID to redo (use 'latest' for the most recent reverted entry)
        id: String,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Show renaming status
    Status {
        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Show renaming history
    History {
        /// Limit number of entries
        #[arg(long)]
        limit: Option<usize>,

        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,

        /// Suppress all output (alias for --preview none)
        #[arg(long)]
        quiet: bool,
    },

    /// Show version information
    Version {
        /// Output format for machine consumption
        #[arg(long, value_enum, default_value = "summary")]
        output: OutputFormat,
    },
}
