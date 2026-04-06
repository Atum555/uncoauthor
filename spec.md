# git-uncoauthor — Spec & Build Instructions

## Overview

A small Rust CLI tool that removes all `Co-authored-by` trailers from every commit between a given base ref and `HEAD`, using an automated interactive rebase.

---

## Usage

```bash
git-uncoauthor <base-ref>
```

- `<base-ref>` — a branch name, tag, or commit SHA. The tool rewrites every commit in the range `<base-ref>..HEAD`.
- The command must be run from inside a git repository with a clean working tree (no uncommitted changes).

### Example

```bash
git-uncoauthor main
```

This rebases all commits from `main` to `HEAD`, stripping any `Co-authored-by: ...` lines from every commit message.

---

## Functional Requirements

### FR-1: CLI Argument Parsing

- Accept exactly one positional argument: the base ref.
- Use `clap` (v4+) with derive macros for argument definition.
- Provide `--help` and `--version` flags automatically.

### FR-2: Shell Completions

- Use `clap_complete` to generate shell completion scripts for **bash**, **zsh**, and **fish**.
- Expose a hidden subcommand (`completions`) or a flag (`--completions <shell>`) that prints the completion script to stdout.
- The completion for `<base-ref>` should suggest local branch names. Register a custom value hint or a shell-specific completion function that runs `git branch --format='%(refname:short)'`.

### FR-3: Pre-flight Checks

Before starting the rebase, the tool must verify:

1. The current directory is inside a git repository (check via `git rev-parse --git-dir`).
2. The working tree is clean — no staged or unstaged changes (check via `git diff --quiet && git diff --cached --quiet`).
3. The provided base ref is valid (check via `git rev-parse --verify <base-ref>`).
4. There is at least one commit in the range `<base-ref>..HEAD`.

If any check fails, print a clear error message to stderr and exit with a non-zero code.

### FR-4: Commit Rewriting via Rebase

Use `git rebase --interactive <base-ref>` driven by two custom editor overrides:

#### Sequence Editor (`GIT_SEQUENCE_EDITOR`)

Replaces every `pick` with `reword` in the rebase todo list. This can be done by:

- Having the binary accept a hidden subcommand (e.g., `git-uncoauthor __sequence-edit <file>`) that reads the todo file, does the replacement, and writes it back.
- Setting `GIT_SEQUENCE_EDITOR="git-uncoauthor __sequence-edit"` in the rebase environment.

#### Commit Message Editor (`GIT_EDITOR`)

Strips `Co-authored-by` lines from the commit message file. This can be done by:

- A second hidden subcommand (e.g., `git-uncoauthor __msg-edit <file>`) that reads the file, removes any line matching `Co-authored-by:` (case-insensitive), cleans up trailing blank lines left behind, and writes it back.
- Setting `GIT_EDITOR="git-uncoauthor __msg-edit"` in the rebase environment.

Both subcommands must be hidden from `--help` output; they are internal implementation details.

### FR-5: Output

- On success, print a summary: `Rewrote N commits. Co-authored-by trailers removed.`
- On failure (rebase conflict, etc.), print the git error output and advise the user to run `git rebase --abort`.

---

## Non-Functional Requirements

### NFR-1: Dependencies

Keep dependencies minimal:

| Crate           | Purpose                            |
| --------------- | ---------------------------------- |
| `clap` (v4+)    | CLI parsing with derive macros     |
| `clap_complete` | Shell completion script generation |

No other external crates should be necessary. Use `std::process::Command` for all git operations — do **not** use `git2`/libgit2 (avoids a heavy native dependency for what is effectively a shell-orchestration tool).

### NFR-2: Error Handling

- Use `std::process::ExitCode` or `std::process::exit` with meaningful codes:
    - `0` — success
    - `1` — pre-flight check failed (bad ref, dirty tree, etc.)
    - `2` — rebase failed
- All errors go to stderr. Normal output goes to stdout.

### NFR-3: Platform Support

- Linux and macOS. Windows support is not required but should not be actively broken.

### NFR-4: Testing

- **Unit tests** for the message-stripping logic (various formats, edge cases like messages with no co-author, multiple co-authors, co-author in the middle of the message body).
- **Integration test** that initialises a temp git repo, creates commits with `Co-authored-by` trailers, runs the tool, and asserts the trailers are gone and commit history is intact.

---

## Co-Author Line Stripping Rules

The message editor must handle these patterns:

```
Co-authored-by: Name <email>
co-authored-by: Name <email>
CO-AUTHORED-BY: Name <email>
```

Matching rule: any line whose trimmed content starts with `co-authored-by:` (case-insensitive) is removed.

After removal, collapse any resulting double-blank-lines at the end of the message into a single trailing newline.

---

## Project Structure

```
git-uncoauthor/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point, CLI dispatch
│   ├── cli.rs           # clap arg definitions + completions subcommand
│   ├── preflight.rs     # Pre-flight validation functions
│   ├── rebase.rs        # Rebase orchestration (env setup, spawning git)
│   └── message.rs       # Commit message parsing and co-author stripping
└── tests/
    └── integration.rs   # End-to-end test with a real git repo
```

---

## Build & Install

```bash
cargo build --release
cp target/release/git-uncoauthor ~/.local/bin/
```

### Generating Shell Completions

```bash
# Bash — add to ~/.bashrc
git-uncoauthor --completions bash >> ~/.bash_completion

# Zsh — add to fpath
git-uncoauthor --completions zsh > ~/.zfunc/_git-uncoauthor

# Fish
git-uncoauthor --completions fish > ~/.config/fish/completions/git-uncoauthor.fish
```

---

## Edge Cases to Handle

1. **No co-author lines in any commit** — rebase still runs, summary says `Rewrote 0 commits` or notes nothing was changed.
2. **Commit message is only a co-author line** — after stripping, the message would be empty. Abort with an error rather than creating an empty commit message.
3. **Rebase conflicts** — let git's error propagate, print advice to `git rebase --abort`.
4. **Detached HEAD** — should still work; the base ref is still valid.
5. **Already rebasing** — pre-flight should detect `.git/rebase-merge` or `.git/rebase-apply` and refuse to start.
