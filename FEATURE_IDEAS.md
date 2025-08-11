some more ideas please:

- [x] 'exclude match' option - a CLI arg you can pass to skip specific matches (e.g. compound words to ignore) - if you're replacing foo with bar and it's accidentally matching on bazFooQux - you could pass --exclude-match bazFooQux
- [x] lock file in .refaktor/ - abort with error if another process is running (for plan/apply/rename)
- homebrew formula
- interactive mode like git add -P for accepting/rejecting individual changes
