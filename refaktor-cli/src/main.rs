use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use refaktor_core::{
    apply_plan, format_history, get_status, redo_refactoring, scan_repository_multi,
    undo_refactoring, write_plan, write_preview, ApplyOptions, Config, History, LockFile, Plan,
    PlanOptions, PreviewFormat, Style,
};
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

mod rename;

/// Returns the default styles used by refaktor CLI
pub(crate) fn get_default_styles() -> Vec<StyleArg> {
    vec![
        StyleArg::Original,
        StyleArg::Snake,
        StyleArg::Kebab,
        StyleArg::Camel,
        StyleArg::Pascal,
        StyleArg::ScreamingSnake,
        StyleArg::Train,          // Include Train-Case in CLI defaults
        StyleArg::ScreamingTrain, // Include ScreamingTrain for ALL-CAPS-PATTERNS
    ]
}

/// Smart search & replace for code and files with case-aware transformations
#[derive(Parser, Debug)]
#[command(name = "refaktor")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    no_color: bool,

    /// Reduce the level of "smart" filtering. Can be repeated up to 3 times.
    /// -u: Don't respect .gitignore files
    /// -uu: Don't respect any ignore files (.gitignore, .ignore, .rgignore, .rfignore), show hidden files
    /// -uuu: Same as -uu, plus treat binary files as text
    #[arg(short = 'u', long = "unrestricted", global = true, action = clap::ArgAction::Count, verbatim_doc_comment)]
    unrestricted: u8,

    /// Run as if started in <path> instead of the current working directory
    #[arg(short = 'C', global = true, value_name = "PATH")]
    directory: Option<PathBuf>,

    /// Automatically initialize .refaktor ignore (repo|local|global)
    #[arg(long, global = true, value_name = "MODE")]
    auto_init: Option<String>,

    /// Disable automatic initialization prompt
    #[arg(long, global = true, conflicts_with = "auto_init")]
    no_auto_init: bool,

    /// Assume yes for all prompts
    #[arg(short = 'y', long = "yes", global = true, env = "REFAKTOR_YES")]
    yes: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a refactoring plan
    Plan {
        /// Old identifier to replace
        old: String,

        /// New identifier to replace with
        new: String,

        /// Paths to search (files or directories). Defaults to current directory
        #[arg(help = "Search paths (files or directories)")]
        paths: Vec<PathBuf>,

        /// Include glob patterns
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Exclude glob patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Respect ignore files (.gitignore, .ignore, .rgignore, .rfignore)
        #[arg(long, default_value_t = true)]
        respect_gitignore: bool,

        /// Don't rename matching files
        #[arg(long = "no-rename-files")]
        no_rename_files: bool,

        /// Don't rename matching directories
        #[arg(long = "no-rename-dirs")]
        no_rename_dirs: bool,

        /// Case styles to exclude from the default set (snake, kebab, camel, pascal, screaming-snake)
        #[arg(
            long,
            value_enum,
            value_delimiter = ',',
            conflicts_with = "only_styles"
        )]
        exclude_styles: Vec<StyleArg>,

        /// Additional case styles to include (title, train, dot)
        #[arg(
            long,
            value_enum,
            value_delimiter = ',',
            conflicts_with = "only_styles"
        )]
        include_styles: Vec<StyleArg>,

        /// Use only these case styles (overrides defaults)
        #[arg(long, value_enum, value_delimiter = ',', conflicts_with_all = ["exclude_styles", "include_styles"])]
        only_styles: Vec<StyleArg>,

        /// Specific matches to exclude (e.g., compound words to ignore)
        #[arg(long, value_delimiter = ',')]
        exclude_match: Vec<String>,

        /// Preview output format (defaults from config if not specified)
        #[arg(long, value_enum)]
        preview_format: Option<PreviewFormatArg>,

        /// Output path for the plan
        #[arg(long, default_value = ".refaktor/plan.json")]
        plan_out: PathBuf,

        /// Only show preview, don't write plan (dry-run)
        #[arg(long)]
        dry_run: bool,
    },

    /// Apply a refactoring plan
    Apply {
        /// Path to plan file
        #[arg(long, conflicts_with = "id")]
        plan: Option<PathBuf>,

        /// History ID to apply
        #[arg(long, conflicts_with = "plan")]
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
    },

    /// Undo a previous refactoring
    Undo {
        /// History ID to undo (use 'latest' for the most recent non-revert entry)
        id: String,
    },

    /// Redo a previously undone refactoring
    Redo {
        /// History ID to redo (use 'latest' for the most recent reverted entry)
        id: String,
    },

    /// Show refactoring status
    Status,

    /// Show refactoring history
    History {
        /// Limit number of entries
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Dry-run mode (alias for plan --dry-run)
    DryRun {
        /// Old identifier to replace
        old: String,

        /// New identifier to replace with
        new: String,

        /// Paths to search (files or directories). Defaults to current directory
        #[arg(help = "Search paths (files or directories)")]
        paths: Vec<PathBuf>,

        /// Include glob patterns
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Exclude glob patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Respect ignore files (.gitignore, .ignore, .rgignore, .rfignore)
        #[arg(long, default_value_t = true)]
        respect_gitignore: bool,

        /// Don't rename matching files
        #[arg(long = "no-rename-files")]
        no_rename_files: bool,

        /// Don't rename matching directories
        #[arg(long = "no-rename-dirs")]
        no_rename_dirs: bool,

        /// Case styles to exclude from the default set (snake, kebab, camel, pascal, screaming-snake)
        #[arg(
            long,
            value_enum,
            value_delimiter = ',',
            conflicts_with = "only_styles"
        )]
        exclude_styles: Vec<StyleArg>,

        /// Additional case styles to include (title, train, dot)
        #[arg(
            long,
            value_enum,
            value_delimiter = ',',
            conflicts_with = "only_styles"
        )]
        include_styles: Vec<StyleArg>,

        /// Use only these case styles (overrides defaults)
        #[arg(long, value_enum, value_delimiter = ',', conflicts_with_all = ["exclude_styles", "include_styles"])]
        only_styles: Vec<StyleArg>,

        /// Specific matches to exclude (e.g., compound words to ignore)
        #[arg(long, value_delimiter = ',')]
        exclude_match: Vec<String>,

        /// Preview output format (defaults from config if not specified)
        #[arg(long, value_enum)]
        preview_format: Option<PreviewFormatArg>,
    },

    /// Initialize refaktor in the current repository
    Init {
        /// Add to .git/info/exclude instead of .gitignore
        #[arg(long, conflicts_with = "global")]
        local: bool,

        /// Add to global git excludes file
        #[arg(long, conflicts_with = "local")]
        global: bool,

        /// Check if .refaktor is ignored (exit 0 if yes, 1 if no)
        #[arg(long, conflicts_with_all = ["local", "global", "configure_global"])]
        check: bool,

        /// Configure global excludes file if it doesn't exist
        #[arg(long, requires = "global")]
        configure_global: bool,
    },

    /// Plan and apply a refactoring in one step (with confirmation)
    Rename {
        /// Old identifier to replace
        old: String,

        /// New identifier to replace with
        new: String,

        /// Paths to search (files or directories). Defaults to current directory
        #[arg(help = "Search paths (files or directories)")]
        paths: Vec<PathBuf>,

        /// Include glob patterns
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Exclude glob patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Don't rename matching files
        #[arg(long = "no-rename-files")]
        no_rename_files: bool,

        /// Don't rename matching directories
        #[arg(long = "no-rename-dirs")]
        no_rename_dirs: bool,

        /// Case styles to exclude from the default set (snake, kebab, camel, pascal, screaming-snake)
        #[arg(
            long,
            value_enum,
            value_delimiter = ',',
            conflicts_with = "only_styles"
        )]
        exclude_styles: Vec<StyleArg>,

        /// Additional case styles to include (title, train, dot)
        #[arg(
            long,
            value_enum,
            value_delimiter = ',',
            conflicts_with = "only_styles"
        )]
        include_styles: Vec<StyleArg>,

        /// Use only these case styles (overrides defaults)
        #[arg(long, value_enum, value_delimiter = ',', conflicts_with_all = ["exclude_styles", "include_styles"])]
        only_styles: Vec<StyleArg>,

        /// Specific matches to exclude (e.g., compound words to ignore)
        #[arg(long, value_delimiter = ',')]
        exclude_match: Vec<String>,

        /// Show preview before confirmation prompt
        #[arg(long, value_enum)]
        preview: Option<PreviewFormatArg>,

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
    },
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum StyleArg {
    Snake,
    Kebab,
    Camel,
    Pascal,
    ScreamingSnake,
    Title,
    Train,
    ScreamingTrain,
    Dot,
    Original,
}

impl From<StyleArg> for Style {
    fn from(arg: StyleArg) -> Self {
        match arg {
            StyleArg::Snake => Self::Snake,
            StyleArg::Kebab => Self::Kebab,
            StyleArg::Camel => Self::Camel,
            StyleArg::Pascal => Self::Pascal,
            StyleArg::ScreamingSnake => Self::ScreamingSnake,
            StyleArg::Title => Self::Title,
            StyleArg::Train => Self::Train,
            StyleArg::ScreamingTrain => Self::ScreamingTrain,
            StyleArg::Dot => Self::Dot,
            StyleArg::Original => Self::Original,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum PreviewFormatArg {
    Table,
    Diff,
    Json,
    Summary,
    None,
}

impl PreviewFormatArg {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(Self::Table),
            "diff" => Some(Self::Diff),
            "json" => Some(Self::Json),
            "summary" => Some(Self::Summary),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

impl From<PreviewFormatArg> for PreviewFormat {
    fn from(arg: PreviewFormatArg) -> Self {
        match arg {
            PreviewFormatArg::Table => Self::Table,
            PreviewFormatArg::Diff => Self::Diff,
            PreviewFormatArg::Json => Self::Json,
            PreviewFormatArg::Summary => Self::Summary,
            PreviewFormatArg::None => Self::Table, // Default to table if None is somehow converted
        }
    }
}

fn main() {
    // Set up signal handler for graceful shutdown
    ctrlc::set_handler(move || {
        eprintln!("\nInterrupted. Cleaning up...");
        process::exit(130); // Standard exit code for SIGINT
    })
    .expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();
    let use_color = !cli.no_color && io::stdout().is_terminal();

    // Handle -C directory flag
    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir)
            .with_context(|| format!("Failed to change to directory: {}", dir.display()))
            .unwrap_or_else(|e| {
                eprintln!("Error: {e:#}");
                process::exit(2);
            });
    }

    // Check if we need to auto-init before running commands that create .refaktor/
    let needs_refaktor_dir = matches!(
        cli.command,
        Commands::Plan { .. }
            | Commands::Apply { .. }
            | Commands::DryRun { .. }
            | Commands::Rename { .. }
    );

    if needs_refaktor_dir && !cli.no_auto_init {
        if let Err(e) = check_and_auto_init(&cli.auto_init, cli.yes) {
            eprintln!("Error during auto-initialization: {e:#}");
            process::exit(2);
        }
    }

    // Load config to get defaults
    let config = Config::load().unwrap_or_default();

    let result = match cli.command {
        Commands::Plan {
            old,
            new,
            paths,
            include,
            exclude,
            respect_gitignore,
            no_rename_files,
            no_rename_dirs,
            exclude_styles,
            include_styles,
            only_styles,
            exclude_match,
            preview_format,
            plan_out,
            dry_run,
        } => {
            // Use preview format from CLI arg or config default
            let format = preview_format
                .map(std::convert::Into::into)
                .unwrap_or_else(|| {
                    PreviewFormat::from_str(&config.defaults.preview_format)
                        .unwrap_or(PreviewFormat::Diff)
                });

            handle_plan(
                &old,
                &new,
                paths,
                include,
                exclude,
                respect_gitignore,
                cli.unrestricted,
                !no_rename_files,
                !no_rename_dirs,
                exclude_styles,
                include_styles,
                only_styles,
                exclude_match,
                format,
                plan_out,
                dry_run,
                use_color,
            )
        },

        Commands::DryRun {
            old,
            new,
            paths,
            include,
            exclude,
            respect_gitignore,
            no_rename_files,
            no_rename_dirs,
            exclude_styles,
            include_styles,
            only_styles,
            exclude_match,
            preview_format,
        } => {
            // Use preview format from CLI arg or config default
            let format = preview_format
                .map(std::convert::Into::into)
                .unwrap_or_else(|| {
                    PreviewFormat::from_str(&config.defaults.preview_format)
                        .unwrap_or(PreviewFormat::Diff)
                });

            handle_plan(
                &old,
                &new,
                paths,
                include,
                exclude,
                respect_gitignore,
                cli.unrestricted,
                !no_rename_files,
                !no_rename_dirs,
                exclude_styles,
                include_styles,
                only_styles,
                exclude_match,
                format,
                PathBuf::from(".refaktor/plan.json"),
                true, // Always dry-run
                use_color,
            )
        },

        Commands::Apply {
            plan,
            id,
            atomic,
            commit,
            force_with_conflicts,
        } => handle_apply(plan, id, atomic, commit, force_with_conflicts),

        Commands::Undo { id } => handle_undo(id),

        Commands::Redo { id } => {
            let refaktor_dir = PathBuf::from(".refaktor");
            redo_refactoring(&id, &refaktor_dir).context("Failed to redo refactoring")
        },

        Commands::Status => handle_status(),

        Commands::History { limit } => handle_history(limit),

        Commands::Init {
            local,
            global,
            check,
            configure_global,
        } => handle_init(local, global, check, configure_global),

        Commands::Rename {
            old,
            new,
            paths,
            include,
            exclude,
            no_rename_files,
            no_rename_dirs,
            exclude_styles,
            include_styles,
            only_styles,
            exclude_match,
            preview,
            commit,
            large,
            force_with_conflicts,
            confirm_collisions,
            rename_root,
            no_rename_root,
            dry_run,
        } => {
            // Use preview format from CLI arg or config default
            let format =
                preview.or_else(|| PreviewFormatArg::from_str(&config.defaults.preview_format));

            rename::handle_rename(
                &old,
                &new,
                paths,
                include,
                exclude,
                cli.unrestricted,
                !no_rename_files,
                !no_rename_dirs,
                exclude_styles,
                include_styles,
                only_styles,
                exclude_match,
                format,
                commit,
                large,
                force_with_conflicts,
                confirm_collisions,
                rename_root,
                no_rename_root,
                dry_run,
                cli.yes,
                use_color,
            )
        },
    };

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {e:#}");

            // Determine exit code based on error type
            let exit_code = if e.to_string().contains("conflict") {
                1 // Conflicts
            } else if e.to_string().contains("invalid") || e.to_string().contains("not found") {
                2 // Invalid input
            } else {
                3 // Internal error
            };

            process::exit(exit_code);
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_plan(
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
    preview_format: PreviewFormat,
    plan_out: PathBuf,
    dry_run: bool,
    use_color: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Use provided paths or default to current directory
    let search_paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
    };

    // Acquire lock
    let refaktor_dir = current_dir.join(".refaktor");
    let _lock = LockFile::acquire(&refaktor_dir)
        .context("Failed to acquire lock for refaktor operation")?;

    // Build the list of styles to use based on exclude, include, and only options
    let styles = {
        if only_styles.is_empty() {
            // Start with the default styles
            let default_styles = get_default_styles();

            // Remove excluded styles from defaults
            let mut active_styles: Vec<StyleArg> = default_styles
                .into_iter()
                .filter(|s| !exclude_styles.contains(s))
                .collect();

            // Add included styles (Title, Train, Dot)
            for style in include_styles {
                if !active_styles.contains(&style) {
                    active_styles.push(style);
                }
            }

            if active_styles.is_empty() {
                eprintln!("Warning: All styles have been excluded, using default styles");
                None // Use default styles
            } else {
                Some(active_styles.into_iter().map(Into::into).collect())
            }
        } else {
            // If --only-styles is specified, use only those styles
            Some(only_styles.into_iter().map(Into::into).collect())
        }
    };

    let options = PlanOptions {
        includes: include,
        excludes: exclude,
        respect_gitignore,
        unrestricted_level: unrestricted.min(3), // Cap at 3 for safety
        styles,
        rename_files,
        rename_dirs,
        rename_root: false, // Default: do not allow root directory renames in plan
        plan_out: plan_out.clone(),
        coerce_separators: refaktor_core::scanner::CoercionMode::Auto, // TODO: make configurable
        exclude_match,
    };

    // Resolve all search paths to absolute paths and canonicalize them
    let resolved_paths: Vec<PathBuf> = search_paths
        .iter()
        .map(|path| {
            let absolute_path = if path.is_absolute() {
                path.clone()
            } else {
                current_dir.join(path)
            };
            // Canonicalize to remove . and .. components
            absolute_path.canonicalize().unwrap_or(absolute_path)
        })
        .collect();

    let plan = scan_repository_multi(&resolved_paths, old, new, &options)
        .context("Failed to scan repository")?;

    // Show preview
    write_preview(&plan, preview_format, Some(use_color)).context("Failed to write preview")?;

    // Write plan unless dry-run
    if !dry_run {
        write_plan(&plan, &plan_out).context("Failed to write plan")?;

        if preview_format != PreviewFormat::Json {
            eprintln!("\nPlan written to: {}", plan_out.display());
        }
    }

    // Check for conflicts and return appropriate exit code
    if let Some(conflicts) = check_for_conflicts(&plan) {
        eprintln!("\nWarning: {conflicts} conflicts detected");
        if !dry_run {
            eprintln!("Use --force-with-conflicts to apply anyway");
        }
        // We don't exit with error here, just warn
    }

    Ok(())
}

const fn check_for_conflicts(_plan: &refaktor_core::Plan) -> Option<usize> {
    // Check if there are any rename conflicts
    // This is a placeholder - would need to check the actual conflicts
    // from the rename module
    None
}

fn handle_apply(
    plan_path: Option<PathBuf>,
    id: Option<String>,
    atomic: bool,
    commit: bool,
    force_with_conflicts: bool,
) -> Result<()> {
    let root = std::env::current_dir().context("Failed to get current directory")?;

    // Acquire lock
    let refaktor_dir = root.join(".refaktor");
    let _lock = LockFile::acquire(&refaktor_dir)
        .context("Failed to acquire lock for refaktor operation")?;

    // Determine which plan to load
    let plan_path = if let Some(path) = plan_path {
        path
    } else if let Some(id) = id {
        // Load from history by ID (placeholder for now)
        eprintln!("Loading plan from history ID {id} not yet implemented");
        return Ok(());
    } else {
        // Default to last plan
        PathBuf::from(".refaktor/plan.json")
    };

    // Load the plan
    let plan_json = std::fs::read_to_string(&plan_path)
        .with_context(|| format!("Failed to read plan from {}", plan_path.display()))?;

    let plan: Plan = serde_json::from_str(&plan_json).context("Failed to parse plan JSON")?;

    // Check for conflicts if not forcing
    if !force_with_conflicts {
        if let Some(conflicts) = check_for_conflicts(&plan) {
            eprintln!(
                "Error: {conflicts} conflicts detected. Use --force-with-conflicts to apply anyway"
            );
            return Err(anyhow!("Conflicts detected"));
        }
    }

    // Set up apply options
    let options = ApplyOptions {
        atomic,
        commit,
        force: force_with_conflicts,
        ..Default::default()
    };

    eprintln!(
        "Applying plan {} ({} edits, {} renames)...",
        plan.id,
        plan.matches.len(),
        plan.renames.len()
    );

    // Apply the plan
    apply_plan(&plan, &options).context("Failed to apply plan")?;

    eprintln!("Plan applied successfully!");

    // Delete the plan.json file after successful apply (only if using default path)
    if plan_path == PathBuf::from(".refaktor/plan.json") {
        if let Err(e) = std::fs::remove_file(&plan_path) {
            eprintln!(
                "Warning: Failed to delete plan file {}: {}",
                plan_path.display(),
                e
            );
        } else {
            eprintln!("Deleted plan file: {}", plan_path.display());
        }
    }

    if commit {
        eprintln!("Changes committed to git");
    }

    Ok(())
}

fn handle_status() -> Result<()> {
    let refaktor_dir = PathBuf::from(".refaktor");
    let status = get_status(&refaktor_dir).context("Failed to get status")?;

    print!("{}", status.format());
    Ok(())
}

fn handle_undo(id: String) -> Result<()> {
    let refaktor_dir = PathBuf::from(".refaktor");

    // Handle "latest" keyword
    let actual_id = if id == "latest" {
        // Load history and get the most recent non-revert entry
        let history = History::load(&refaktor_dir).context("Failed to load history")?;
        let entries = history.list_entries(None);

        // Find the most recent non-revert entry
        entries
            .iter()
            .find(|e| e.revert_of.is_none())
            .map(|e| e.id.clone())
            .ok_or_else(|| anyhow!("No entries to undo"))?
    } else {
        id
    };

    undo_refactoring(&actual_id, &refaktor_dir).context("Failed to undo refactoring")
}

fn handle_history(limit: Option<usize>) -> Result<()> {
    let refaktor_dir = PathBuf::from(".refaktor");
    let history = History::load(&refaktor_dir).context("Failed to load history")?;

    let entries = history.list_entries(limit);
    let formatted = format_history(&entries, false)?;

    println!("{formatted}");
    Ok(())
}

fn is_refaktor_ignored() -> Result<bool> {
    // Check if .refaktor is already ignored in any ignore file

    // 1. Check .gitignore
    if let Ok(content) = std::fs::read_to_string(".gitignore") {
        if is_pattern_in_content(&content) {
            return Ok(true);
        }
    }

    // 2. Check .git/info/exclude (if in git repo)
    if let Ok(git_dir) = find_git_dir() {
        let exclude_path = git_dir.join("info").join("exclude");
        if let Ok(content) = std::fs::read_to_string(exclude_path) {
            if is_pattern_in_content(&content) {
                return Ok(true);
            }
        }
    }

    // 3. Check global excludes
    if let Ok(global_path) = get_global_excludes_path() {
        if let Ok(content) = std::fs::read_to_string(global_path) {
            if is_pattern_in_content(&content) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn is_pattern_in_content(content: &str) -> bool {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .any(|line| {
            line == ".refaktor"
                || line == ".refaktor/"
                || line == "/.refaktor"
                || line == "/.refaktor/"
        })
}

fn check_and_auto_init(auto_init: &Option<String>, yes: bool) -> Result<()> {
    // If .refaktor is already ignored, nothing to do
    if is_refaktor_ignored()? {
        return Ok(());
    }

    // Check if .refaktor is tracked in git
    if is_in_git_repo()? && is_file_tracked(".refaktor")? {
        eprintln!("\n⚠ Error: .refaktor directory is already tracked by git.");
        eprintln!("  Please run: git rm -r --cached .refaktor");
        eprintln!("  Then run your command again.");
        process::exit(1);
    }

    // Determine init mode
    let mode = if let Some(mode) = auto_init {
        // Explicit mode specified
        match mode.as_str() {
            "repo" => InitMode::Repo,
            "local" => InitMode::Local,
            "global" => InitMode::Global,
            _ => {
                eprintln!("Invalid auto-init mode: {mode}. Use repo, local, or global.");
                process::exit(2);
            },
        }
    } else if yes {
        // -y flag: default to repo
        InitMode::Repo
    } else if io::stdin().is_terminal() && io::stdout().is_terminal() {
        // Interactive mode: show prompt
        prompt_for_init()?
    } else {
        // Non-interactive: do nothing
        return Ok(());
    };

    // Perform the initialization
    match mode {
        InitMode::Repo => do_init(false, false, false)?,
        InitMode::Local => do_init(true, false, false)?,
        InitMode::Global => do_init(false, true, false)?,
        InitMode::Skip => return Ok(()),
    }

    Ok(())
}

#[derive(Debug)]
enum InitMode {
    Repo,
    Local,
    Global,
    Skip,
}

fn prompt_for_init() -> Result<InitMode> {
    use std::io::Write;

    eprintln!("\nRefaktor uses .refaktor/ for plans, backups, and history.");
    eprintln!("Ignore it now?");
    eprintln!(
        "  [Y] Repo .gitignore   [l] Local .git/info/exclude   [g] Global excludesfile   [n] No"
    );
    eprint!("Choice (Y/l/g/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim().to_lowercase();

    match choice.as_str() {
        "" | "y" | "yes" => Ok(InitMode::Repo),
        "l" | "local" => Ok(InitMode::Local),
        "g" | "global" => Ok(InitMode::Global),
        "n" | "no" => Ok(InitMode::Skip),
        _ => {
            eprintln!("Invalid choice. Please enter Y, l, g, or n.");
            prompt_for_init()
        },
    }
}

fn do_init(local: bool, global: bool, _configure_global: bool) -> Result<()> {
    // This is the core init logic
    const REFAKTOR_PATTERN: &str = ".refaktor/";

    // Determine which file to modify
    let target_path = if global {
        // Get global git excludes file
        get_global_excludes_path()?
    } else if local {
        // Use .git/info/exclude
        let git_dir = find_git_dir()?;
        git_dir.join("info").join("exclude")
    } else {
        // Default: .gitignore in current directory
        PathBuf::from(".gitignore")
    };

    // Check if pattern already exists in file
    let existing_content = if target_path.exists() {
        std::fs::read_to_string(&target_path)
            .with_context(|| format!("Failed to read {}", target_path.display()))?
    } else {
        String::new()
    };

    // Check if pattern already exists
    if is_pattern_in_content(&existing_content) {
        eprintln!(".refaktor is already ignored in {}", target_path.display());
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    // Append pattern to file
    let mut content = existing_content;

    // Add newline if file doesn't end with one
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    // Add comment and pattern
    if !content.is_empty() {
        content.push('\n'); // Blank line before our section
    }
    content.push_str("# Refaktor workspace\n");
    content.push_str(REFAKTOR_PATTERN);
    content.push('\n');

    // Write atomically
    use std::io::Write;
    let temp_path = target_path.with_extension("tmp");
    {
        let mut file = std::fs::File::create(&temp_path)
            .with_context(|| format!("Failed to create temporary file {}", temp_path.display()))?;
        file.write_all(content.as_bytes())
            .context("Failed to write content")?;
        file.sync_all().context("Failed to sync file")?;
    }

    std::fs::rename(&temp_path, &target_path).with_context(|| {
        format!(
            "Failed to rename {} to {}",
            temp_path.display(),
            target_path.display()
        )
    })?;

    eprintln!("✓ Added .refaktor/ to {}", target_path.display());

    Ok(())
}

fn configure_global_excludes() -> Result<()> {
    // Check if global excludes file is already configured
    let output = std::process::Command::new("git")
        .args(["config", "--global", "core.excludesFile"])
        .output()
        .context("Failed to run git config")?;

    if output.status.success() {
        let existing = String::from_utf8(output.stdout)?.trim().to_string();
        if !existing.is_empty() {
            eprintln!("Global excludes file already configured: {existing}");
            return Ok(());
        }
    }

    // Set default global excludes path
    let default_path = if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("git").join("ignore")
    } else {
        return Err(anyhow!("Could not determine config directory"));
    };

    eprintln!(
        "Setting global excludes file to: {}",
        default_path.display()
    );

    std::process::Command::new("git")
        .args([
            "config",
            "--global",
            "core.excludesFile",
            &default_path.to_string_lossy(),
        ])
        .output()
        .context("Failed to set git config")?;

    Ok(())
}

fn handle_init(local: bool, global: bool, check: bool, configure_global: bool) -> Result<()> {
    // Check mode: just verify if .refaktor is ignored
    if check {
        if is_refaktor_ignored()? {
            eprintln!(".refaktor is properly ignored");
            return Ok(());
        }
        eprintln!(".refaktor is NOT ignored");
        process::exit(1);
    }

    // Handle configure_global flag
    if configure_global && global {
        configure_global_excludes()?;
    }

    // Use the common init logic
    do_init(local, global, configure_global)?;

    // Check if .refaktor is tracked by git (only if not using --global)
    if !global && is_in_git_repo()? && is_file_tracked(".refaktor")? {
        eprintln!("\n⚠ Warning: .refaktor directory is already tracked by git.");
        eprintln!("  You may want to run: git rm -r --cached .refaktor");
    }

    Ok(())
}

fn find_git_dir() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("Failed to run git rev-parse")?;

    if !output.status.success() {
        return Err(anyhow!("Not in a git repository"));
    }

    let git_dir = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();

    Ok(PathBuf::from(git_dir))
}

fn get_global_excludes_path() -> Result<PathBuf> {
    // First check if core.excludesFile is configured
    let output = std::process::Command::new("git")
        .args(["config", "--global", "core.excludesFile"])
        .output()
        .context("Failed to run git config")?;

    if output.status.success() {
        let path = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string();

        if !path.is_empty() {
            // Expand ~ if present
            if let Some(stripped) = path.strip_prefix("~/") {
                if let Some(home) = dirs::home_dir() {
                    return Ok(home.join(stripped));
                }
            }
            return Ok(PathBuf::from(path));
        }
    }

    // Use default location
    if let Some(config_dir) = dirs::config_dir() {
        Ok(config_dir.join("git").join("ignore"))
    } else {
        Err(anyhow!("Could not determine global git excludes path"))
    }
}

fn is_in_git_repo() -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to run git rev-parse")?;

    Ok(output.status.success())
}

fn is_file_tracked(path: &str) -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["ls-files", "--error-unmatch", path])
        .output()
        .context("Failed to run git ls-files")?;

    Ok(output.status.success())
}

// Generate shell completions
pub fn generate_completions<G: clap_complete::Generator>(
    gen: G,
    cmd: &mut clap::Command,
    name: &str,
    out_dir: &std::path::Path,
) -> Result<()> {
    use clap_complete::generate_to;
    use std::fs;

    fs::create_dir_all(out_dir)?;
    let path = generate_to(gen, cmd, name, out_dir)?;
    println!("Generated completion file: {}", path.display());
    Ok(())
}
