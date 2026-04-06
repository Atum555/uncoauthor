use std::process::Command;

/// Run the automated interactive rebase. Returns the number of commits
/// that had co-author lines removed.
pub fn run_rebase(base_ref: &str) -> Result<usize, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot determine own path: {e}"))?;
    let exe_str = exe.display().to_string();

    let counter_file =
        std::env::temp_dir().join(format!("uncoauthor-counter-{}", std::process::id()));
    std::fs::write(&counter_file, "0").map_err(|e| format!("cannot create counter file: {e}"))?;

    let seq_editor = format!("\"{}\" __sequence-edit", exe_str);
    let msg_editor = format!("\"{}\" __msg-edit", exe_str);

    let output = Command::new("git")
        .args(["rebase", "--interactive", base_ref])
        .env("GIT_SEQUENCE_EDITOR", &seq_editor)
        .env("GIT_EDITOR", &msg_editor)
        .env("GIT_UNCOAUTHOR_COUNTER_FILE", &counter_file)
        .output()
        .map_err(|e| format!("failed to run git rebase: {e}"))?;

    let rewritten: usize = std::fs::read_to_string(&counter_file)
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(0);

    let _ = std::fs::remove_file(&counter_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(rewritten)
}
