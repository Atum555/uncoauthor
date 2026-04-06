#[derive(Debug)]
pub enum StripError {
    EmptyMessage,
}

impl std::fmt::Display for StripError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StripError::EmptyMessage => {
                write!(f, "commit message would be empty after stripping co-author lines")
            }
        }
    }
}

/// Strip all `Co-authored-by` lines (case-insensitive) from a commit message.
/// Returns the cleaned message and the number of removed lines.
/// Errors if the resulting message would be empty.
pub fn strip_coauthors(msg: &str) -> Result<(String, usize), StripError> {
    let lines: Vec<&str> = msg.lines().collect();
    let mut kept: Vec<&str> = Vec::new();
    let mut removed = 0usize;

    for line in &lines {
        if line.trim_start().to_ascii_lowercase().starts_with("co-authored-by:") {
            removed += 1;
        } else {
            kept.push(line);
        }
    }

    // Trim trailing blank lines
    while kept.last().map_or(false, |l| l.trim().is_empty()) {
        kept.pop();
    }

    if kept.is_empty() || kept.iter().all(|l| l.trim().is_empty()) {
        return Err(StripError::EmptyMessage);
    }

    let mut result = kept.join("\n");
    result.push('\n');
    Ok((result, removed))
}

/// Rewrite a rebase todo file, replacing `pick` with `reword`.
pub fn rewrite_todo(contents: &str) -> String {
    contents
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("pick ") {
                let indent = &line[..line.len() - trimmed.len()];
                format!("{}reword {}", indent, &trimmed["pick ".len()..])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_coauthor_at_end() {
        let msg = "feat: add feature\n\nCo-authored-by: Alice <alice@example.com>\n";
        let (cleaned, count) = strip_coauthors(msg).unwrap();
        assert_eq!(count, 1);
        assert_eq!(cleaned, "feat: add feature\n");
        assert!(!cleaned.contains("Co-authored-by"));
    }

    #[test]
    fn multiple_coauthors() {
        let msg = "fix: bug\n\nCo-authored-by: Alice <a@b>\nCo-authored-by: Bob <b@c>\n";
        let (cleaned, count) = strip_coauthors(msg).unwrap();
        assert_eq!(count, 2);
        assert_eq!(cleaned, "fix: bug\n");
    }

    #[test]
    fn case_insensitive() {
        let msg = "msg\n\nco-authored-by: A <a@b>\nCO-AUTHORED-BY: B <b@c>\n";
        let (cleaned, count) = strip_coauthors(msg).unwrap();
        assert_eq!(count, 2);
        assert_eq!(cleaned, "msg\n");
    }

    #[test]
    fn no_coauthors() {
        let msg = "feat: nothing special\n\nSigned-off-by: Dev <d@e>\n";
        let (cleaned, count) = strip_coauthors(msg).unwrap();
        assert_eq!(count, 0);
        assert_eq!(cleaned, msg);
    }

    #[test]
    fn coauthor_in_middle_of_body() {
        let msg = "title\n\nsome body text\nCo-authored-by: A <a@b>\nmore text\n";
        let (cleaned, count) = strip_coauthors(msg).unwrap();
        assert_eq!(count, 1);
        assert_eq!(cleaned, "title\n\nsome body text\nmore text\n");
    }

    #[test]
    fn only_coauthor_line_errors() {
        let msg = "Co-authored-by: A <a@b>\n";
        assert!(matches!(strip_coauthors(msg), Err(StripError::EmptyMessage)));
    }

    #[test]
    fn trailing_blank_lines_collapsed() {
        let msg = "title\n\nbody\n\nCo-authored-by: A <a@b>\n\n\n";
        let (cleaned, count) = strip_coauthors(msg).unwrap();
        assert_eq!(count, 1);
        assert_eq!(cleaned, "title\n\nbody\n");
    }

    #[test]
    fn rewrite_todo_basic() {
        let todo = "pick abc123 first commit\npick def456 second commit\n";
        let result = rewrite_todo(todo);
        assert_eq!(result, "reword abc123 first commit\nreword def456 second commit\n");
    }

    #[test]
    fn rewrite_todo_preserves_comments() {
        let todo = "pick abc123 msg\n# comment line\npick def456 msg2\n";
        let result = rewrite_todo(todo);
        assert!(result.contains("reword abc123"));
        assert!(result.contains("# comment line"));
        assert!(result.contains("reword def456"));
    }
}
