## CRITICAL: YOU ARE AN AUTONOMOUS AGENT. DO NOT STOP WORKING UNTIL ALL TASKMASTER TASKS ARE COMPLETE. DO NOT EVER "PAUSE" AFTER A TASK IS COMPLETED AND WAIT FOR THE USER TO START YOU AGAIN. START THE NEXT TASK IMMEDIATELY.

## **Refaktor** – Smart Search & Replace for Code and Files

Summary: Case-aware and file/directory aware search and replace to refactor with built-in undo/redo

# Released as

- VS Code / Cursor extension
- CLI tool
- MCP server

# Features

- Uses ripgrep for all search operations
- Can automatically detect the case of your search string, e.g. “old-name” = kebab case
- By default, it will also convert your search string into the following variations:
  - snake_case
  - kebab-case
  - camelCase
  - PascalCase
  - UPPER_SNAKE_CASE
- if your replace string is “new-name”
  - old-name => new-name
  - old_name => new_name
  - oldName => newName
  - OldName => NewName
  - OLD_NAME => NEW_NAME
  - plus plain uppercase/lowercase
- And if you have any files or directories with that same name, they get renamed too:
  - /path/to/old-name.txt => /path/to/new-name.txt
  - /path/to/old_name/foo.txt => /path/to/new_name/foo.txt
- Supports all the same UI options as VS Code’s current search and replace:
  - Search field with case sensitivity toggle, regex - PLUS the new case options - e.g. dropdown list where you can toggle checkboxes for different cases
  - Replace field
  - files to include
  - files to exclude
  - toggle auto-exclude .gitignore, etc.
- search results will be organized into subgroups for each case + a group for files / directories
- built-in undo/redo
  - A full history of search/replace actions stored per project. e.g. .refaktor/history.json
    - Prune history automatically when the file reaches ~100kb or so
  - Tracked separately to git since these actions are often taken while you are working on a refactor with lots of unstaged changes.
    - It can be a pain to commit everything and get the git worktree clean before doing big renames
  - Store the exact options used and all the replacements that were made (a “changeset”)
  - A change can easily be undone by clicking a ‘revert’ icon in that row in the history pane, or via a TUI (CLI)
  - Clearly show which replacements can be reverted and which have conflicts (if the line was changed)

## Task Master AI Instructions

**Import Task Master's development workflow commands and guidelines, treat as if import is in the main CLAUDE.md file.**
@./.taskmaster/CLAUDE.md

## CRITICAL: YOU ARE AN AUTONOMOUS AGENT. DO NOT STOP WORKING UNTIL ALL TASKMASTER TASKS ARE COMPLETE. DO NOT EVER "PAUSE" AFTER A TASK IS COMPLETED AND WAIT FOR THE USER TO START YOU AGAIN. START THE NEXT TASK IMMEDIATELY.

The ONLY reason you should stop working and pause is if you have an urgent question for the user that requires their attention. Otherwise, continue working on all tasks, even after compacting the conversation history, continue working in a loop until all tasks are complete.
