# uncoauthor

A small Rust CLI tool that removes all `Co-authored-by` trailers from every commit in a given range, using an automated interactive rebase.

## Why?

Tools like GitHub Copilot and AI pair-programming assistants often insert `Co-authored-by` trailers into commit messages. If you want a clean history without those attributions, `uncoauthor` strips them in one command.

## Quick Start

```bash
# Strip co-author trailers from all commits since main
uncoauthor main

# Since a specific commit
uncoauthor abc1234

# Since a tag
uncoauthor v1.0.0
```

## Installation

### From source

```bash
cargo build --release
cp target/release/uncoauthor ~/.local/bin/
```

Make sure `~/.local/bin` is in your `PATH`.

### Shell Completions

```bash
# Bash — append to ~/.bashrc or ~/.bash_completion
uncoauthor --completions bash >> ~/.bash_completion

# Zsh — place in your fpath
uncoauthor --completions zsh > ~/.zfunc/_uncoauthor

# Fish
uncoauthor --completions fish > ~/.config/fish/completions/uncoauthor.fish
```

## Usage

```
uncoauthor <base-ref>
```

| Argument     | Description                                                   |
| ------------ | ------------------------------------------------------------- |
| `<base-ref>` | Branch name, tag, or commit SHA. Rewrites `<base-ref>..HEAD`. |

| Flag                    | Description                             |
| ----------------------- | --------------------------------------- |
| `--completions <SHELL>` | Print shell completion script to stdout |
| `--help`                | Show help                               |
| `--version`             | Show version                            |

### Requirements

- Must be run inside a git repository
- Working tree must be clean (no staged or unstaged changes)
- No rebase already in progress

### Exit Codes

| Code | Meaning                                             |
| ---- | --------------------------------------------------- |
| `0`  | Success                                             |
| `1`  | Pre-flight check failed (bad ref, dirty tree, etc.) |
| `2`  | Rebase failed (conflict, empty message, etc.)       |

## How It Works

1. **Pre-flight checks** — verifies you're in a git repo, the tree is clean, the ref is valid, and there are commits to rewrite.
2. **Automated interactive rebase** — runs `git rebase --interactive <base-ref>` with two custom editor overrides:
    - **Sequence editor** — rewrites every `pick` to `reword` in the todo list.
    - **Message editor** — strips any line matching `Co-authored-by:` (case-insensitive), then collapses trailing blank lines.
3. **Summary** — prints how many commits had trailers removed.

If the rebase fails (e.g., due to conflicts), the tool prints git's error output and advises you to run `git rebase --abort`.

## Development

```bash
# Run all tests (unit + integration)
cargo test

# Build release binary
cargo build --release
```

See [docs/](docs/) for architecture and design details.

## License

MIT
