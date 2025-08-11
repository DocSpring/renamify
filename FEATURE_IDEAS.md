some more ideas please:

- 'exclude match' option - a CLI arg you can pass to skip specific matches (e.g. compound words to ignore) - if you're replacing foo with bar and it's accidentally matching on bazFooQux - you could pass --exclude-match bazFooQux
- lock file in .refaktor/ - abort with error if another process is running (for plan/apply/rename)
- homebrew formula
