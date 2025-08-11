use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use refaktor_core::{
    apply_plan, ApplyOptions, Plan, PlanOptions, PreviewFormat, scan_repository, write_plan, 
    write_preview, Style, History, format_history, get_status, undo_refactoring, redo_refactoring,
};
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process;

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
    /// -uu: Don't respect any ignore files, show hidden files  
    /// -uuu: Same as -uu, plus treat binary files as text
    #[arg(short = 'u', long = "unrestricted", global = true, action = clap::ArgAction::Count)]
    unrestricted: u8,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a refactoring plan
    Plan {
        /// Old identifier to replace
        old: String,

        /// New identifier to replace with
        new: String,

        /// Include glob patterns
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Exclude glob patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Respect .gitignore files
        #[arg(long, default_value_t = true)]
        respect_gitignore: bool,

        /// Don't rename matching files
        #[arg(long = "no-rename-files")]
        no_rename_files: bool,

        /// Don't rename matching directories
        #[arg(long = "no-rename-dirs")]
        no_rename_dirs: bool,

        /// Naming styles to use
        #[arg(long, value_enum, value_delimiter = ',')]
        styles: Vec<StyleArg>,

        /// Preview output format
        #[arg(long, value_enum, default_value = "table")]
        preview_format: PreviewFormatArg,

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
        /// History ID to undo
        id: String,
    },

    /// Redo a previously undone refactoring
    Redo {
        /// History ID to redo
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

        /// Include glob patterns
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,

        /// Exclude glob patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Respect .gitignore files
        #[arg(long, default_value_t = true)]
        respect_gitignore: bool,

        /// Don't rename matching files
        #[arg(long = "no-rename-files")]
        no_rename_files: bool,

        /// Don't rename matching directories
        #[arg(long = "no-rename-dirs")]
        no_rename_dirs: bool,

        /// Naming styles to use
        #[arg(long, value_enum, value_delimiter = ',')]
        styles: Vec<StyleArg>,

        /// Preview output format
        #[arg(long, value_enum, default_value = "table")]
        preview_format: PreviewFormatArg,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StyleArg {
    Snake,
    Kebab,
    Camel,
    Pascal,
    ScreamingSnake,
    Title,
    Train,
    Dot,
}

impl From<StyleArg> for Style {
    fn from(arg: StyleArg) -> Self {
        match arg {
            StyleArg::Snake => Style::Snake,
            StyleArg::Kebab => Style::Kebab,
            StyleArg::Camel => Style::Camel,
            StyleArg::Pascal => Style::Pascal,
            StyleArg::ScreamingSnake => Style::ScreamingSnake,
            StyleArg::Title => Style::Title,
            StyleArg::Train => Style::Train,
            StyleArg::Dot => Style::Dot,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PreviewFormatArg {
    Table,
    Diff,
    Json,
}

impl From<PreviewFormatArg> for PreviewFormat {
    fn from(arg: PreviewFormatArg) -> Self {
        match arg {
            PreviewFormatArg::Table => PreviewFormat::Table,
            PreviewFormatArg::Diff => PreviewFormat::Diff,
            PreviewFormatArg::Json => PreviewFormat::Json,
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

    let result = match cli.command {
        Commands::Plan {
            old,
            new,
            include,
            exclude,
            respect_gitignore,
            no_rename_files,
            no_rename_dirs,
            styles,
            preview_format,
            plan_out,
            dry_run,
        } => handle_plan(
            &old,
            &new,
            include,
            exclude,
            respect_gitignore,
            cli.unrestricted,
            !no_rename_files,
            !no_rename_dirs,
            styles,
            preview_format.into(),
            plan_out,
            dry_run,
            use_color,
        ),

        Commands::DryRun {
            old,
            new,
            include,
            exclude,
            respect_gitignore,
            no_rename_files,
            no_rename_dirs,
            styles,
            preview_format,
        } => handle_plan(
            &old,
            &new,
            include,
            exclude,
            respect_gitignore,
            cli.unrestricted,
            !no_rename_files,
            !no_rename_dirs,
            styles,
            preview_format.into(),
            PathBuf::from(".refaktor/plan.json"),
            true, // Always dry-run
            use_color,
        ),

        Commands::Apply { 
            plan,
            id,
            atomic,
            commit,
            force_with_conflicts,
        } => handle_apply(
            plan,
            id,
            atomic,
            commit,
            force_with_conflicts,
        ),

        Commands::Undo { id } => {
            let refaktor_dir = PathBuf::from(".refaktor");
            undo_refactoring(&id, &refaktor_dir)
                .context("Failed to undo refactoring")
        }

        Commands::Redo { id } => {
            let refaktor_dir = PathBuf::from(".refaktor");
            redo_refactoring(&id, &refaktor_dir)
                .context("Failed to redo refactoring")
        }

        Commands::Status => handle_status(),

        Commands::History { limit } => handle_history(limit),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {:#}", e);
            
            // Determine exit code based on error type
            let exit_code = if e.to_string().contains("conflict") {
                1 // Conflicts
            } else if e.to_string().contains("invalid") || e.to_string().contains("not found") {
                2 // Invalid input
            } else {
                3 // Internal error
            };
            
            process::exit(exit_code);
        }
    }
}

fn handle_plan(
    old: &str,
    new: &str,
    include: Vec<String>,
    exclude: Vec<String>,
    respect_gitignore: bool,
    unrestricted: u8,
    rename_files: bool,
    rename_dirs: bool,
    styles: Vec<StyleArg>,
    preview_format: PreviewFormat,
    plan_out: PathBuf,
    dry_run: bool,
    use_color: bool,
) -> Result<()> {
    let root = std::env::current_dir().context("Failed to get current directory")?;

    let styles = if styles.is_empty() {
        None
    } else {
        Some(styles.into_iter().map(Into::into).collect())
    };

    let options = PlanOptions {
        includes: include,
        excludes: exclude,
        respect_gitignore,
        unrestricted_level: unrestricted.min(3),  // Cap at 3 for safety
        styles,
        rename_files,
        rename_dirs,
        plan_out: plan_out.clone(),
    };

    // Scan the repository
    let plan = scan_repository(&root, old, new, &options)
        .context("Failed to scan repository")?;

    // Show preview
    write_preview(&plan, preview_format, Some(use_color))
        .context("Failed to write preview")?;

    // Write plan unless dry-run
    if !dry_run {
        write_plan(&plan, &plan_out)
            .context("Failed to write plan")?;
        
        if preview_format != PreviewFormat::Json {
            eprintln!("\nPlan written to: {}", plan_out.display());
        }
    }

    // Check for conflicts and return appropriate exit code
    if let Some(conflicts) = check_for_conflicts(&plan) {
        eprintln!("\nWarning: {} conflicts detected", conflicts);
        if !dry_run {
            eprintln!("Use --force-with-conflicts to apply anyway");
        }
        // We don't exit with error here, just warn
    }

    Ok(())
}

fn check_for_conflicts(_plan: &refaktor_core::Plan) -> Option<usize> {
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
    // Determine which plan to load
    let plan_path = if let Some(path) = plan_path {
        path
    } else if let Some(id) = id {
        // Load from history by ID (placeholder for now)
        eprintln!("Loading plan from history ID {} not yet implemented", id);
        return Ok(());
    } else {
        // Default to last plan
        PathBuf::from(".refaktor/plan.json")
    };
    
    // Load the plan
    let plan_json = std::fs::read_to_string(&plan_path)
        .with_context(|| format!("Failed to read plan from {}", plan_path.display()))?;
    
    let plan: Plan = serde_json::from_str(&plan_json)
        .context("Failed to parse plan JSON")?;
    
    // Check for conflicts if not forcing
    if !force_with_conflicts {
        if let Some(conflicts) = check_for_conflicts(&plan) {
            eprintln!("Error: {} conflicts detected. Use --force-with-conflicts to apply anyway", conflicts);
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
    
    eprintln!("Applying plan {} ({} edits, {} renames)...", 
        plan.id, 
        plan.matches.len(),
        plan.renames.len()
    );
    
    // Apply the plan
    apply_plan(&plan, &options)
        .context("Failed to apply plan")?;
    
    eprintln!("Plan applied successfully!");
    
    if commit {
        eprintln!("Changes committed to git");
    }
    
    Ok(())
}

fn handle_status() -> Result<()> {
    let refaktor_dir = PathBuf::from(".refaktor");
    let status = get_status(&refaktor_dir)
        .context("Failed to get status")?;
    
    print!("{}", status.format());
    Ok(())
}

fn handle_history(limit: Option<usize>) -> Result<()> {
    let refaktor_dir = PathBuf::from(".refaktor");
    let history = History::load(&refaktor_dir)
        .context("Failed to load history")?;
    
    let entries = history.list_entries(limit);
    let formatted = format_history(&entries, false)?;
    
    println!("{}", formatted);
    Ok(())
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