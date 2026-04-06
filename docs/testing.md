# Testing

## Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# With output
cargo test -- --nocapture
```

## Unit Tests

Located in `src/message.rs` under `#[cfg(test)]`. These test the pure stripping and todo-rewriting logic:

| Test                          | What it verifies                                   |
|-------------------------------|----------------------------------------------------|
| `single_coauthor_at_end`      | Removes a single trailer, reports count = 1        |
| `multiple_coauthors`          | Removes multiple trailers                          |
| `case_insensitive`            | Matches `co-authored-by`, `CO-AUTHORED-BY`, etc.   |
| `no_coauthors`                | Returns message unchanged, count = 0               |
| `coauthor_in_middle_of_body`  | Removes mid-body trailer, preserves surrounding text |
| `only_coauthor_line_errors`   | Returns `Err(EmptyMessage)` when nothing would remain |
| `trailing_blank_lines_collapsed` | Collapses trailing blanks after removal          |
| `rewrite_todo_basic`          | Replaces `pick` with `reword`                      |
| `rewrite_todo_preserves_comments` | Leaves `#` comment lines untouched             |

## Integration Tests

Located in `tests/integration.rs`. Each test creates a temporary git repository, makes commits, runs the built binary, and asserts on the results.

| Test                    | What it verifies                                          |
|-------------------------|-----------------------------------------------------------|
| `test_strips_coauthors` | Removes trailers, reports correct rewrite count, clean log |
| `test_no_coauthors`     | Reports 0 rewrites when no trailers exist                 |
| `test_invalid_ref`      | Exit code 1, error mentions invalid ref                   |
| `test_dirty_tree`       | Exit code 1, error mentions clean working tree            |
| `test_no_commits_in_range` | Exit code 1, error mentions no commits                 |

### How Integration Tests Work

The `TestRepo` helper struct:
1. Creates a temp directory with a unique name (test name + PID)
2. Initializes a git repo with dummy user config
3. Provides `commit()` and `run_tool()` methods
4. Cleans up the temp directory on drop

The binary path is resolved relative to the test binary location, so `cargo test` builds and tests the latest code automatically.
