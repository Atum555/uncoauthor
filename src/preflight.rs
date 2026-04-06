use std::path::Path;
use std::process::Command;

/// Run all pre-flight checks. Returns the number of commits in range on success.
pub fn run_preflight(base_ref: &str) -> Result<usize, String> {
    check_git_repo()?;
    check_not_rebasing()?;
    check_clean_tree()?;
    check_valid_ref(base_ref)?;
    count_commits(base_ref)
}

fn check_git_repo() -> Result<(), String> {
    let status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if !status.success() {
        return Err("not inside a git repository".into());
    }
    Ok(())
}

fn check_not_rebasing() -> Result<(), String> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if Path::new(&git_dir).join("rebase-merge").exists()
        || Path::new(&git_dir).join("rebase-apply").exists()
    {
        return Err(
            "a rebase is already in progress. Run `git rebase --abort` or `git rebase --continue` first".into(),
        );
    }
    Ok(())
}

fn check_clean_tree() -> Result<(), String> {
    let unstaged = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;

    let staged = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if !unstaged.success() || !staged.success() {
        return Err("working tree is not clean. Please commit or stash your changes".into());
    }
    Ok(())
}

fn check_valid_ref(base_ref: &str) -> Result<(), String> {
    let status = Command::new("git")
        .args(["rev-parse", "--verify", base_ref])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if !status.success() {
        return Err(format!("invalid base ref: `{base_ref}`"));
    }
    Ok(())
}

fn count_commits(base_ref: &str) -> Result<usize, String> {
    let output = Command::new("git")
        .args(["rev-list", "--count", &format!("{base_ref}..HEAD")])
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if !output.status.success() {
        return Err(format!("failed to count commits in range `{base_ref}..HEAD`"));
    }

    let count: usize = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|e| format!("failed to parse commit count: {e}"))?;

    if count == 0 {
        return Err(format!("no commits in range `{base_ref}..HEAD`"));
    }
    Ok(count)
}
