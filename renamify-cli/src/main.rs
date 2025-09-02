use anyhow::{anyhow, Context, Result};
use clap::Parser;
use renamify_core::{Config, OutputFormatter, Preview, VersionResult};
use std::io::{self, BufRead, IsTerminal};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod apply;
mod cli;
mod history;
mod plan;
mod redo;
mod rename;
mod replace;
mod status;
mod undo;

#[cfg(test)]
mod test_lock_signals;

// Import from our new cli module
use cli::{Cli, Commands, OutputFormat, PreviewArg};

fn main() {
    // Set up signal handler for graceful shutdown (both SIGINT and SIGTERM)
    let interrupted = Arc::new(AtomicBool::new(false));

    // Handle SIGINT (Ctrl-C)
    let interrupted_clone = Arc::clone(&interrupted);
    ctrlc::set_handler(move || {
        eprintln!("\nReceived SIGINT. Cleaning up...");
        interrupted_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting SIGINT handler");

    // Handle SIGTERM (sent by VS Code)
    let interrupted_clone = Arc::clone(&interrupted);
    unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGTERM, move || {
            eprintln!("\nReceived SIGTERM. Cleaning up...");
            interrupted_clone.store(true, Ordering::SeqCst);
        })
        .expect("Error setting SIGTERM handler");
    }

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
            | Commands::Rename { .. }
            | Commands::Replace { .. }
            | Commands::Search { .. }
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
            search,
            replace,
            paths,
            filter,
            rename_files,
            styles,
            exclude_match,
            exclude_matching_lines,
            preview,
            fixed_table_width,
            plan_out,
            dry_run,
            acronyms,
            atomic,
            output,
            quiet,
        } => {
            // Use preview format from CLI arg or config default (unless JSON output)
            let format = if output == OutputFormat::Json {
                None // No preview for JSON output
            } else {
                Some(preview.map(std::convert::Into::into).unwrap_or_else(|| {
                    Preview::from_str(&config.defaults.preview_format).unwrap_or(Preview::Diff)
                }))
            };

            plan::handle_plan(
                &search,
                &replace,
                paths,
                filter.include,
                filter.exclude,
                filter.respect_gitignore,
                cli.unrestricted,
                !rename_files.no_rename_files && !rename_files.no_rename_paths,
                !rename_files.no_rename_dirs && !rename_files.no_rename_paths,
                styles.exclude_styles,
                styles.include_styles,
                styles.only_styles,
                exclude_match,
                exclude_matching_lines,
                format,
                fixed_table_width,
                plan_out,
                dry_run,
                use_color,
                acronyms.no_acronyms,
                acronyms.include_acronyms,
                acronyms.exclude_acronyms,
                acronyms.only_acronyms,
                atomic,
                output,
                quiet,
                false, // regex flag - not used in Plan command
            )
        },

        Commands::Search {
            term,
            paths,
            include,
            exclude,
            rename_files,
            rename_dirs,
            styles,
            exclude_matching_lines,
            preview,
            fixed_table_width,
            acronyms,
            output,
            quiet,
        } => {
            // Use preview format from CLI arg or default to matches for search (unless JSON output)
            let format = if output == OutputFormat::Json {
                None // No preview for JSON output
            } else {
                Some(preview.map(std::convert::Into::into).unwrap_or_else(|| {
                    // For search, default to matches instead of diff
                    let config_format = Preview::from_str(&config.defaults.preview_format)
                        .unwrap_or(Preview::Matches);
                    if config_format == Preview::Diff {
                        Preview::Matches // If config has diff, use matches for search
                    } else {
                        config_format
                    }
                }))
            };

            // Call plan handler with empty replacement string and dry_run=true
            plan::handle_plan(
                &term,
                "", // Empty replacement string for search
                paths,
                include,
                exclude,
                true, // respect_gitignore (use default true for search)
                cli.unrestricted,
                rename_files,
                rename_dirs,
                styles.exclude_styles,
                styles.include_styles,
                styles.only_styles,
                vec![], // exclude_match not needed for search
                exclude_matching_lines,
                format,
                fixed_table_width,
                PathBuf::from(".renamify/plan.json"),
                true, // Always dry-run for search
                use_color,
                acronyms.no_acronyms,
                acronyms.include_acronyms,
                acronyms.exclude_acronyms,
                acronyms.only_acronyms,
                cli::args::AtomicArgs {
                    atomic_identifiers: false,
                    atomic_search: false,
                    atomic_replace: false,
                    no_atomic_identifiers: false,
                    no_atomic_search: false,
                    no_atomic_replace: false,
                },
                output,
                quiet,
                false, // regex flag - not used in Search command
            )
        },

        Commands::Apply {
            id,
            commit,
            force_with_conflicts,
            output,
            quiet,
        } => apply::handle_apply(id, commit, force_with_conflicts, output, quiet),

        Commands::Undo { id, output, quiet } => undo::handle_undo(&id, output, quiet),

        Commands::Redo { id, output, quiet } => redo::handle_redo(&id, output, quiet),

        Commands::Status { output, quiet } => status::handle_status(output, quiet),

        Commands::History {
            limit,
            output,
            quiet,
        } => history::handle_history(limit, output, quiet),

        Commands::Init {
            local,
            global,
            check,
            configure_global,
        } => handle_init(local, global, check, configure_global),

        Commands::Version { output } => handle_version(output),

        Commands::TestLock { delay } => handle_test_lock(delay, Arc::clone(&interrupted)),

        Commands::Rename {
            search,
            replace,
            paths,
            filter,
            rename_files,
            styles,
            exclude_match,
            exclude_matching_lines,
            preview,
            commit,
            large,
            force_with_conflicts,
            confirm_collisions,
            rename_root,
            no_rename_root,
            dry_run,
            acronyms,
            atomic,
            output,
            quiet,
        } => {
            // Use preview format from CLI arg or config default
            let format = preview.or_else(|| PreviewArg::from_str(&config.defaults.preview_format));

            rename::handle_rename(
                &search,
                &replace,
                paths,
                filter.include,
                filter.exclude,
                cli.unrestricted,
                !rename_files.no_rename_files && !rename_files.no_rename_paths,
                !rename_files.no_rename_dirs && !rename_files.no_rename_paths,
                styles.exclude_styles,
                styles.include_styles,
                styles.only_styles,
                exclude_match,
                exclude_matching_lines,
                format,
                commit,
                large,
                force_with_conflicts,
                confirm_collisions,
                rename_root,
                no_rename_root,
                dry_run,
                acronyms.no_acronyms,
                acronyms.include_acronyms,
                acronyms.exclude_acronyms,
                acronyms.only_acronyms,
                atomic,
                cli.yes,
                use_color,
                output,
                quiet,
            )
        },

        Commands::Replace {
            pattern,
            replacement,
            paths,
            no_regex,
            filter,
            rename_files,
            exclude_matching_lines,
            preview,
            commit,
            large,
            force_with_conflicts,
            dry_run,
            yes,
            output,
            quiet,
        } => {
            // Use preview format from CLI arg or config default
            let format = preview.or_else(|| PreviewArg::from_str(&config.defaults.preview_format));

            replace::handle_replace(
                &pattern,
                &replacement,
                paths,
                no_regex,
                filter.include,
                filter.exclude,
                cli.unrestricted,
                !rename_files.no_rename_files && !rename_files.no_rename_paths,
                !rename_files.no_rename_dirs && !rename_files.no_rename_paths,
                exclude_matching_lines,
                format,
                commit,
                large,
                force_with_conflicts,
                dry_run,
                yes || cli.yes,
                use_color,
                output,
                quiet,
            )
        },
    };

    // Check if we were interrupted during execution
    if interrupted.load(Ordering::SeqCst) {
        // We were interrupted - exit gracefully to allow Drop destructors to run
        eprintln!("Operation interrupted, cleaning up...");
        std::process::exit(130);
    }

    match result {
        Ok(()) => std::process::exit(0),
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

            std::process::exit(exit_code);
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
    prompt_for_init_with_input(&mut io::stdin())
}

fn prompt_for_init_with_input<R: io::Read>(reader: &mut R) -> Result<InitMode> {
    use std::io::Write;

    eprintln!("\nRenamify uses .renamify/ for plans, backups, and history.");
    eprintln!("Ignore it now?");
    eprintln!(
        "  [Y] Repo .gitignore   [l] Local .git/info/exclude   [g] Global excludesfile   [n] No"
    );
    eprint!("Choice (Y/l/g/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::BufReader::new(reader).read_line(&mut input)?;
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

fn handle_version(output: OutputFormat) -> Result<()> {
    let version_result = VersionResult {
        name: "renamify".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let formatted = match output {
        OutputFormat::Json => version_result.format_json(),
        OutputFormat::Summary => version_result.format_summary(),
    };

    println!("{}", formatted);
    Ok(())
}

fn handle_test_lock(delay: u64, interrupted: Arc<AtomicBool>) -> Result<()> {
    use renamify_core::LockFile;
    use std::thread;
    use std::time::Duration;

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let renamify_dir = current_dir.join(".renamify");

    // Ensure .renamify directory exists
    if !renamify_dir.exists() {
        std::fs::create_dir_all(&renamify_dir).context("Failed to create .renamify directory")?;
    }

    eprintln!("Acquiring lock...");
    let _lock = LockFile::acquire(&renamify_dir)
        .context("Failed to acquire lock for test-lock operation")?;

    eprintln!("Lock acquired. Sleeping for {}ms...", delay);

    // Check for interruption periodically during sleep
    let sleep_interval = 100; // Check every 100ms
    let mut remaining = delay;

    while remaining > 0 && !interrupted.load(Ordering::SeqCst) {
        let sleep_time = std::cmp::min(remaining, sleep_interval);
        thread::sleep(Duration::from_millis(sleep_time));
        remaining = remaining.saturating_sub(sleep_time);
    }

    if interrupted.load(Ordering::SeqCst) {
        eprintln!("Interrupted during sleep, releasing lock...");
        return Ok(()); // Let Drop handle cleanup
    }

    eprintln!("Sleep complete. Lock will be released automatically on exit.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap_complete::Shell;
    use tempfile::TempDir;

    #[test]
    fn test_generate_completions_bash() {
        use clap::CommandFactory;
        let temp_dir = TempDir::new().unwrap();
        let mut cmd = <Cli as CommandFactory>::command();

        let result = generate_completions(Shell::Bash, &mut cmd, "renamify", temp_dir.path());

        assert!(result.is_ok());

        // Check that the completion file was created
        let completion_file = temp_dir.path().join("renamify.bash");
        assert!(completion_file.exists());

        // Read and verify the content has bash completion markers
        let content = std::fs::read_to_string(completion_file).unwrap();
        assert!(content.contains("complete"));
        assert!(content.contains("renamify"));
    }

    #[test]
    fn test_generate_completions_zsh() {
        use clap::CommandFactory;
        let temp_dir = TempDir::new().unwrap();
        let mut cmd = <Cli as CommandFactory>::command();

        let result = generate_completions(Shell::Zsh, &mut cmd, "renamify", temp_dir.path());

        assert!(result.is_ok());

        // Check that the completion file was created
        let completion_file = temp_dir.path().join("_renamify");
        assert!(completion_file.exists());

        // Read and verify the content has zsh completion markers
        let content = std::fs::read_to_string(completion_file).unwrap();
        assert!(content.contains("#compdef"));
        assert!(content.contains("renamify"));
    }

    #[test]
    fn test_generate_completions_fish() {
        use clap::CommandFactory;
        let temp_dir = TempDir::new().unwrap();
        let mut cmd = <Cli as CommandFactory>::command();

        let result = generate_completions(Shell::Fish, &mut cmd, "renamify", temp_dir.path());

        assert!(result.is_ok());

        // Check that the completion file was created
        let completion_file = temp_dir.path().join("renamify.fish");
        assert!(completion_file.exists());

        // Read and verify the content has fish completion markers
        let content = std::fs::read_to_string(completion_file).unwrap();
        assert!(content.contains("complete"));
        assert!(content.contains("-c renamify"));
    }

    #[test]
    fn test_generate_completions_creates_directory() {
        use clap::CommandFactory;
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir");
        let mut cmd = <Cli as CommandFactory>::command();

        // Directory doesn't exist yet
        assert!(!nested_path.exists());

        let result = generate_completions(Shell::Bash, &mut cmd, "renamify", &nested_path);

        assert!(result.is_ok());

        // Directory was created
        assert!(nested_path.exists());

        // File was created in the directory
        let completion_file = nested_path.join("renamify.bash");
        assert!(completion_file.exists());
    }

    #[test]
    fn test_prompt_for_init_repo() {
        let input = b"y\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Repo));

        let input = b"Y\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Repo));

        let input = b"yes\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Repo));

        let input = b"\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Repo));
    }

    #[test]
    fn test_prompt_for_init_local() {
        let input = b"l\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Local));

        let input = b"local\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Local));
    }

    #[test]
    fn test_prompt_for_init_global() {
        let input = b"g\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Global));

        let input = b"global\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Global));
    }

    #[test]
    fn test_prompt_for_init_skip() {
        let input = b"n\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Skip));

        let input = b"no\n";
        let result = prompt_for_init_with_input(&mut &input[..]).unwrap();
        assert!(matches!(result, InitMode::Skip));
    }
}
