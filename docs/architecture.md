# Architecture

## Overview

`uncoauthor` is a shell-orchestration tool. It does not link against libgit2 — all git operations use `std::process::Command` to invoke the `git` CLI. This keeps the dependency footprint minimal and the build fast.

## Module Map

```
src/
├── main.rs        Entry point and CLI dispatch
├── cli.rs         Argument definitions (clap derive)
├── preflight.rs   Pre-rebase validation
├── rebase.rs      Rebase orchestration
└── message.rs     Commit message transformation (pure logic)
```

### `main.rs` — Entry Point

Parses CLI args and dispatches to one of three paths:

1. **`--completions <shell>`** — prints shell completion script and exits.
2. **Hidden subcommand** (`__sequence-edit` or `__msg-edit`) — used internally by the rebase process (see below).
3. **Main flow** — runs preflight checks, then the rebase.

### `cli.rs` — Argument Definitions

Uses `clap` v4 derive macros. The `Cli` struct has:

- `base_ref: Option<String>` — the positional argument (optional because hidden subcommands don't need it).
- `completions: Option<Shell>` — the `--completions` flag.
- `command: Option<InternalCommand>` — hidden subcommands for the self-reinvocation pattern.

### `preflight.rs` — Pre-flight Checks

`run_preflight(base_ref) -> Result<usize, String>` runs five checks in sequence, short-circuiting on the first failure:

1. Inside a git repository (`git rev-parse --git-dir`)
2. No rebase already in progress (checks for `.git/rebase-merge` or `.git/rebase-apply`)
3. Clean working tree (`git diff --quiet && git diff --cached --quiet`)
4. Valid base ref (`git rev-parse --verify <ref>`)
5. At least one commit in range (`git rev-list --count <ref>..HEAD`)

Returns the commit count on success.

### `rebase.rs` — Rebase Orchestration

`run_rebase(base_ref) -> Result<usize, String>` is the core of the tool. It:

1. Determines the path to its own executable (`std::env::current_exe()`).
2. Creates a temporary counter file to track how many commits had trailers removed.
3. Runs `git rebase --interactive <base-ref>` with two environment overrides:
    - `GIT_SEQUENCE_EDITOR` → `"<self> __sequence-edit"` (replaces `pick` with `reword`)
    - `GIT_EDITOR` → `"<self> __msg-edit"` (strips co-author lines)
4. Reads the counter file and returns the count.

### `message.rs` — Message Transformation

Pure functions with no side effects — easy to unit test.

- **`strip_coauthors(msg) -> Result<(String, usize), StripError>`** — removes lines starting with `co-authored-by:` (case-insensitive), collapses trailing blanks, errors if the message would be empty.
- **`rewrite_todo(contents) -> String`** — replaces `pick` with `reword` in rebase todo files.

## Self-Reinvocation Pattern

The tool uses a **self-reinvocation** pattern for the rebase editors. Instead of writing separate scripts or helper binaries, the main binary accepts hidden subcommands (`__sequence-edit` and `__msg-edit`) that git invokes during the rebase.

```
User runs:  uncoauthor main
                │
                ▼
         preflight checks
                │
                ▼
    git rebase --interactive main
      GIT_SEQUENCE_EDITOR="uncoauthor __sequence-edit"
      GIT_EDITOR="uncoauthor __msg-edit"
                │
                ├──▶ git calls: uncoauthor __sequence-edit /tmp/todo
                │        (rewrites pick → reword)
                │
                └──▶ git calls: uncoauthor __msg-edit /tmp/COMMIT_EDITMSG
                         (strips Co-authored-by lines, repeats per commit)
```

## Counter File

To track how many commits actually had trailers removed (vs. how many were in the range), the tool uses a temporary file:

1. `run_rebase` creates a temp file initialized to `"0"` and passes its path via `GIT_UNCOAUTHOR_COUNTER_FILE`.
2. Each `__msg-edit` invocation increments the counter if it removed any lines.
3. After the rebase completes, `run_rebase` reads the counter and cleans up the file.

This is safe because git processes rebase commits sequentially — no race conditions.
