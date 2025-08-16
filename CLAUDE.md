## CRITICAL: YOU ARE AN AUTONOMOUS AGENT. DO NOT STOP WORKING UNTIL ALL TASKMASTER TASKS ARE COMPLETE. DO NOT EVER "PAUSE" AFTER A TASK IS COMPLETED AND WAIT FOR THE USER TO START YOU AGAIN. START THE NEXT TASK IMMEDIATELY.

The ONLY reason you should stop is if you have a blocking question for the user. Otherwise keep executing the workflow loop until done.

---

# Renamify - CLAUDE Agent Guide

Smart case-aware search and replace across code and files with atomic apply and undo.

## Import Task Master rules

Treat this file as if it imports Task Master rules:
@./.taskmaster/CLAUDE.md

## Project background

- Greenfield, unreleased. No backwards compatibility. Delete and tidy old code immediately.
- Goal: ship a production-ready v1.0.0
- Prioritize developer UX and ergonomics
- Key insight: developers need cross-case content replacement plus file and directory renames in one atomic operation with preview and history.

## Project Creator / Maintainer

The company who is sponsoring the development of this project and maintaining it is: DocSpring.
They should be mentioned in any package files, copyright notices, etc.

## Deliverables

- Rust core library and CLI (`renamify-core`, `renamify-cli`)
- VS Code extension (TypeScript) that shells out to the CLI
- MCP server (TypeScript) that wraps the CLI and exposes tools for Cursor or other MCP clients

## Core principles

- Safety first: plan, preview, atomic apply, rollback, history
- Case intelligence: generate all variants for a token
- One-pass scan: compile a single regex and scan once
- Git-aware safety. Use git for safe points and optional commits. Do not touch `.git` internals directly
- DX over legacy. No backward compatibility constraints

## Technologies

- Rust for core and CLI
- Search stack: ripgrep ecosystem crates and friends
  - `ignore` for .gitignore and fast dir walking
  - `globset` for include and exclude globs
  - `regex-automata` and `aho-corasick` for matching
  - `bstr` for fast byte string operations
- JSON for plan and history
- TypeScript for VS Code extension and MCP wrapper
- Optional Node bindings in future via `napi-rs` if needed

## Functional scope

- Case styles: snake_case, kebab-case, camelCase, PascalCase, SCREAMING_SNAKE_CASE, Title Case, Train-Case, dot.case
- Plan: generate all old variants, map to new variants, create a single search program, scan once, write `.renamify/plan.json`
- Apply: update file contents, then rename files and directories, all atomically
- Undo and redo: `.renamify/history.json` with checksums
- Conflicts: re-validate hunks, auto-resolve simple formatting shifts, stop on real conflicts unless forced
- Respect ignore files by default (`.gitignore`, `.ignore`, `.rgignore`, `.rnignore`), allow include and exclude globs
- Exclude binary files by default

## Non-goals for v1

- No AST or language-semantic renaming
- Only VS Code IDE integration
- Local execution only
- No telemetry by default

## Repo layout

- `renamify-core` - core logic
- `renamify-cli` - CLI frontend
- `.claude/agents` - orchestrator, executor, checker role specs
- `.taskmaster` - Task Master config, templates, and docs
- `.taskmaster/docs/prd.txt` - PRD used by Task Master

## Agent roles and behavior

- Orchestrator: plan tasks, maintain dependency order, call Task Master commands
- Executor: implement code and tests, propose diffs, write files, update docs
- Checker: verify acceptance criteria, run linters and tests, measure coverage
- Always keep moving. After finishing a task, immediately pick the next task
- Ask the user only on blockers that cannot be resolved from repo context

## Definition of done

- Code compiles on macOS and Linux
- CI passes: format, clippy, tests
- 100 percent coverage for core operations
- Plan and apply work on sample repos with preview, atomic apply, and undo

## Quality bars

- Clippy warnings are errors
- `rustfmt` enforced
- Property tests for case conversions and boundaries
- Snapshot tests for plans and diffs
- Fuzz tests for regex generation to prevent backtracking issues
- Cross platform tests including Windows path edge cases

## CLI contract

Binary: `renamify`

Commands:

- `renamify plan <old> <new> [opts]`
  - `--include` `--exclude` `--respect-gitignore` (default true, respects all ignore files)
  - `--rename-files` `--rename-dirs` (default true)
  - `--styles=<list>`
  - `--preview table|diff|json`
  - `--plan-out`
  - `-u/-uu/-uuu` (unrestricted levels to control ignore file handling)
- `renamify apply [--plan PATH | --id ID] [--atomic true] [--commit]`
- `renamify undo <id>`
- `renamify redo <id>`
- `renamify history [--limit N]`
- `renamify status`

Exit codes:

- 0 success
- 1 conflicts
- 2 invalid input
- 3 internal error

## Data formats

`.renamify/plan.json`

- `{ id, created_at, old, new, styles[], includes[], excludes[], matches[], renames[], stats, version }`

`.renamify/history.json`

- append only with checksums and revert info

## Search and plan algorithm

1. Detect input case of `<old>` and `<new>`
2. Generate all `old_variant -> new_variant` mappings
3. Build a combined regex or Aho-Corasick automaton with boundary heuristics
4. One walk of the repo using `ignore` with ignore files honored (`.gitignore`, `.ignore`, `.rgignore`, `.rnignore`)
5. For each match, capture file, line, byte range, and preview text
6. For file and directory names, detect and schedule renames with depth ordering
7. Emit `plan.json` and fast summary stats

Boundary rules

- Avoid partial token matches inside larger identifiers unless intended
- Handle camel and Pascal transitions by checking case transitions in code
- Post-filter to enforce boundaries where the regex engine cannot

## Ignore file handling

Renamify respects multiple ignore file formats:

- `.gitignore` - Standard Git ignore patterns
- `.ignore` - Generic ignore file (compatible with ripgrep)
- `.rgignore` - Ripgrep-specific ignore patterns
- `.rnignore` - Renamify-specific ignore patterns (useful for excluding files from renaming without affecting Git)

The unrestricted levels (`-u` flag) control ignore behavior:

- Level 0 (default): Respects all ignore files, skips hidden files
- Level 1 (`-u`): Ignores `.gitignore` but respects `.ignore`, `.rgignore`, `.rnignore`
- Level 2 (`-uu`): Ignores all ignore files, shows hidden files
- Level 3 (`-uuu`): Same as level 2, plus treats binary files as text

## Apply engine

- Optional safe point: `git status` check and a user opt in to create a lightweight commit
- Create temp backups for all files to edit
- Apply content edits first, then renames depth first to avoid collisions
- On failure, restore backups and revert any partial renames
- Verify result hashes match the plan or mark conflicts

## VS Code extension

- Commands: Plan, Preview, Apply, Undo
- UI: QuickPick for options, webview diff with rename badges, progress with cancel
- Implementation: spawn CLI with JSON over stdio. Handle path to bundled or user provided binary

## MCP server

- Node TypeScript wrapper around the CLI
- Tools: `plan`, `apply`, `undo`, `history`, `preview`
- Installed via `npx`, expects `renamify` on PATH

## Coding standards

- Rust edition per `rust-toolchain.toml`
- CLI: `clap` for args, `anyhow` for error context, `serde` for JSON, `tracing` for logs
- Tests live beside code, plus `tests/` for integration
- Use `tempfile` for backups and test sandboxes

## Workflow runbook for the agent

1. Begin next task and keep going
   - Implement tasks in order unless dependencies dictate otherwise
2. After each task
   - Run tests and linters
   - Update docs if anything changed
   - Commit with a clear message
3. Only pause for a blocking user question

## User preferences

- Keep responses short and terse in chats
- Use simple hyphen, not the em dash
- If showing apt commands, always include `-y`
- Prefer actionable diffs and files over long explanations

## Testing Guidelines

- **ALWAYS use the --dry-run flag when testing the renamify CLI** to avoid creating unwanted plan files and modifications
- When running test commands with renamify, use: `./target/debug/renamify plan ... --dry-run`
- This prevents the creation of `.renamify/plan.json` files during testing

### CI Self-Hosting Testing

- **Use "renamed_renaming_tool" NOT the alternative protected string in tests**
- The alternative protected string is only allowed in files matching `.rnignore` entries:
  - `.github/workflows/`
  - `docs/src/content/docs/index.mdx`
  - `docs/src/assets/case-studies/`
  - `docs/src/content/docs/case-studies/`
- All other test files should use "renamed_renaming_tool" as the target replacement string
- This prevents CI failures when renamify tests itself and ensures clean self-hosting testing

## DO NOT REDIRECT STDERR

"2>&1" causes problems in Claude Code. DO NOT USE IT.

## UPDATE THIS FILE!

Remember to update your own CLAUDE.md file with corrections and improvements while you are working.
This file is not set in stone, it is a living document that you should update as you work to make you more effective. Your context window will regularly reset when the conversation history is "compacted", so this file is your core memory.

## CRITICAL REMINDER: YOU ARE AN AUTONOMOUS AGENT. DO NOT STOP WORKING UNTIL ALL TASKMASTER TASKS ARE COMPLETE. DO NOT EVER "PAUSE" AFTER A TASK IS COMPLETED AND WAIT FOR THE USER TO START YOU AGAIN. START THE NEXT TASK IMMEDIATELY.
