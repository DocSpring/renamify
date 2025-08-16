use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use renamify_core::{Config, Preview, Style};
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

mod apply;
mod history;
mod plan;
mod redo;
mod rename;
mod status;
mod undo;

/// Smart search & replace for code and files with case-aware transformations
#[derive(Parser, Debug)]
#[command(name = "renamify")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    no_color: bool,

    /// Reduce the level of "smart" filtering. Can be repeated up to 3 times.
    /// -u: Don't respect .gitignore files
    /// -uu: Don't respect any ignore files (.gitignore, .ignore, .rgignore, .rnignore), include hidden files
    /// -uuu: Same as -uu, plus treat binary files as text
    #[arg(short = 'u', long = "unrestricted", global = true, action = clap::ArgAction::Count, verbatim_doc_comment)]
    unrestricted: u8,

    /// Run as if started in <path> instead of the current working directory
    #[arg(short = 'C', global = true, value_name = "PATH")]
    directory: Option<PathBuf>,

    /// Automatically initialize .renamify ignore (repo|local|global)
    #[arg(long, global = true, value_name = "MODE")]
    auto_init: Option<String>,

    /// Disable automatic initialization prompt
    #[arg(long, global = true, conflicts_with = "auto_init")]
    no_auto_init: bool,

    /// Assume yes for all prompts
    #[arg(short = 'y', long = "yes", global = true, env = "RENAMIFY_YES")]
    yes: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a renaming plan
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

        /// Respect ignore files (.gitignore, .ignore, .rgignore, .rnignore)
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

        /// Disable acronym detection (treat CLI, API, etc. as regular words)
        #[arg(long)]
        no_acronyms: bool,

        /// Additional acronyms to recognize (comma-separated, e.g., "AWS,GCP,K8S")
        #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
        include_acronyms: Vec<String>,

        /// Default acronyms to exclude (comma-separated, e.g., "ID,UI")
        #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
        exclude_acronyms: Vec<String>,

        /// Use only these acronyms (replaces default list)
        #[arg(long, value_delimiter = ',', conflicts_with_all = ["include_acronyms", "exclude_acronyms"])]
        only_acronyms: Vec<String>,
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
    },

    /// Undo a previous renaming
    Undo {
        /// History ID to undo (use 'latest' for the most recent non-revert entry)
        id: String,
    },

    /// Redo a previously undone renaming
    Redo {
        /// History ID to redo (use 'latest' for the most recent reverted entry)
        id: String,
    },

    /// Show renaming status
    Status,

    /// Show renaming history
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

        /// Respect ignore files (.gitignore, .ignore, .rgignore, .rnignore)
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
        preview: Option<PreviewArg>,

        /// Use fixed column widths for table output (useful in CI environments or other non-TTY use cases)
        #[arg(long)]
        fixed_table_width: bool,

        /// Disable acronym detection (treat CLI, API, etc. as regular words)
        #[arg(long)]
        no_acronyms: bool,

        /// Additional acronyms to recognize (comma-separated, e.g., "AWS,GCP,K8S")
        #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
        include_acronyms: Vec<String>,

        /// Default acronyms to exclude (comma-separated, e.g., "ID,UI")
        #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
        exclude_acronyms: Vec<String>,

        /// Use only these acronyms (replaces default list)
        #[arg(long, value_delimiter = ',', conflicts_with_all = ["include_acronyms", "exclude_acronyms"])]
        only_acronyms: Vec<String>,
    },

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

    /// Plan and apply a renaming in one step (with confirmation)
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

        /// Disable acronym detection (treat CLI, API, etc. as regular words)
        #[arg(long)]
        no_acronyms: bool,

        /// Additional acronyms to recognize (comma-separated, e.g., "AWS,GCP,K8S")
        #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
        include_acronyms: Vec<String>,

        /// Default acronyms to exclude (comma-separated, e.g., "ID,UI")
        #[arg(long, value_delimiter = ',', conflicts_with = "only_acronyms")]
        exclude_acronyms: Vec<String>,

        /// Use only these acronyms (replaces default list)
        #[arg(long, value_delimiter = ',', conflicts_with_all = ["include_acronyms", "exclude_acronyms"])]
        only_acronyms: Vec<String>,
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
enum PreviewArg {
    Table,
    Diff,
    Json,
    Summary,
    None,
}

impl PreviewArg {
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

impl From<PreviewArg> for Preview {
    fn from(arg: PreviewArg) -> Self {
        match arg {
            PreviewArg::Table => Self::Table,
            PreviewArg::Diff => Self::Diff,
            PreviewArg::Json => Self::Json,
            PreviewArg::Summary => Self::Summary,
            PreviewArg::None => Self::Table, // Default to table if None is somehow converted
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

    // Check if we need to auto-init before running commands that create .renamify/
    let needs_renamify_dir = matches!(
        cli.command,
        Commands::Plan { .. }
            | Commands::Apply { .. }
            | Commands::DryRun { .. }
            | Commands::Rename { .. }
    );

    if needs_renamify_dir && !cli.no_auto_init {
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
            preview,
            fixed_table_width,
            plan_out,
            dry_run,
            no_acronyms,
            include_acronyms,
            exclude_acronyms,
            only_acronyms,
        } => {
            // Use preview format from CLI arg or config default
            let format = preview.map(std::convert::Into::into).unwrap_or_else(|| {
                Preview::from_str(&config.defaults.preview_format).unwrap_or(Preview::Diff)
            });

            plan::handle_plan(
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
                Some(format),
                fixed_table_width,
                plan_out,
                dry_run,
                use_color,
                no_acronyms,
                include_acronyms,
                exclude_acronyms,
                only_acronyms,
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
            preview,
            fixed_table_width,
            no_acronyms,
            include_acronyms,
            exclude_acronyms,
            only_acronyms,
        } => {
            // Use preview format from CLI arg or config default
            let format = preview.map(std::convert::Into::into).unwrap_or_else(|| {
                Preview::from_str(&config.defaults.preview_format).unwrap_or(Preview::Diff)
            });

            plan::handle_plan(
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
                Some(format),
                fixed_table_width,
                PathBuf::from(".renamify/plan.json"),
                true, // Always dry-run
                use_color,
                no_acronyms,
                include_acronyms,
                exclude_acronyms,
                only_acronyms,
            )
        },

        Commands::Apply {
            id,
            atomic: _,
            commit,
            force_with_conflicts,
        } => apply::handle_apply(id, commit, force_with_conflicts),

        Commands::Undo { id } => undo::handle_undo(&id),

        Commands::Redo { id } => redo::handle_redo(&id),

        Commands::Status => status::handle_status(),

        Commands::History { limit } => history::handle_history(limit),

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
            no_acronyms,
            include_acronyms,
            exclude_acronyms,
            only_acronyms,
        } => {
            // Use preview format from CLI arg or config default
            let format = preview.or_else(|| PreviewArg::from_str(&config.defaults.preview_format));

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
                no_acronyms,
                include_acronyms,
                exclude_acronyms,
                only_acronyms,
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

fn is_renamify_ignored() -> Result<bool> {
    // Check if .renamify is already ignored in any ignore file

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
            line == ".renamify"
                || line == ".renamify/"
                || line == "/.renamify"
                || line == "/.renamify/"
        })
}

fn check_and_auto_init(auto_init: &Option<String>, yes: bool) -> Result<()> {
    // If .renamify is already ignored, nothing to do
    if is_renamify_ignored()? {
        return Ok(());
    }

    // Check if .renamify is tracked in git
    if is_in_git_repo()? && is_file_tracked(".renamify")? {
        eprintln!("\n⚠ Error: .renamify directory is already tracked by git.");
        eprintln!("  Please run: git rm -r --cached .renamify");
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

    eprintln!("\nRenamify uses .renamify/ for plans, backups, and history.");
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
    const RENAMIFY_PATTERN: &str = ".renamify/";

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
        eprintln!(".renamify is already ignored in {}", target_path.display());
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
    content.push_str("# Renamify workspace\n");
    content.push_str(RENAMIFY_PATTERN);
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

    eprintln!("✓ Added .renamify/ to {}", target_path.display());

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
    // Check mode: just verify if .renamify is ignored
    if check {
        if is_renamify_ignored()? {
            eprintln!(".renamify is properly ignored");
            return Ok(());
        }
        eprintln!(".renamify is NOT ignored");
        process::exit(1);
    }

    // Handle configure_global flag
    if configure_global && global {
        configure_global_excludes()?;
    }

    // Use the common init logic
    do_init(local, global, configure_global)?;

    // Check if .renamify is tracked by git (only if not using --global)
    if !global && is_in_git_repo()? && is_file_tracked(".renamify")? {
        eprintln!("\n⚠ Warning: .renamify directory is already tracked by git.");
        eprintln!("  You may want to run: git rm -r --cached .renamify");
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
