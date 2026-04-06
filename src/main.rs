mod cli;
mod message;
mod preflight;
mod rebase;

use clap::Parser;
use cli::{Cli, InternalCommand};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Handle completions
    if let Some(shell) = cli.completions {
        cli::print_completions(shell);
        return ExitCode::SUCCESS;
    }

    // Handle hidden subcommands
    if let Some(cmd) = cli.command {
        return match cmd {
            InternalCommand::SequenceEdit { file } => handle_sequence_edit(&file),
            InternalCommand::MsgEdit { file } => handle_msg_edit(&file),
        };
    }

    // Main flow: require base_ref (prompt interactively if missing)
    let base_ref = match cli.base_ref {
        Some(r) => r,
        None => match pick_branch() {
            Ok(branch) => branch,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(1);
            }
        },
    };

    // Pre-flight checks
    if let Err(msg) = preflight::run_preflight(&base_ref) {
        eprintln!("error: {msg}");
        return ExitCode::from(1);
    }

    // Rebase
    match rebase::run_rebase(&base_ref) {
        Ok(rewritten) => {
            println!("Rewrote {rewritten} commits. Co-authored-by trailers removed.");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: rebase failed\n{err}");
            eprintln!("hint: run `git rebase --abort` to undo");
            ExitCode::from(2)
        }
    }
}

fn handle_sequence_edit(file: &str) -> ExitCode {
    let contents = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading {file}: {e}");
            return ExitCode::from(1);
        }
    };
    let rewritten = message::rewrite_todo(&contents);
    match std::fs::write(file, rewritten) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error writing {file}: {e}");
            ExitCode::from(1)
        }
    }
}

fn handle_msg_edit(file: &str) -> ExitCode {
    let contents = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading {file}: {e}");
            return ExitCode::from(1);
        }
    };
    match message::strip_coauthors(&contents) {
        Ok((cleaned, removed_count)) => {
            if removed_count > 0 {
                if let Ok(counter_path) = std::env::var("GIT_UNCOAUTHOR_COUNTER_FILE") {
                    let current: usize = std::fs::read_to_string(&counter_path)
                        .unwrap_or_default()
                        .trim()
                        .parse()
                        .unwrap_or(0);
                    let _ = std::fs::write(&counter_path, (current + 1).to_string());
                }
            }
            match std::fs::write(file, cleaned) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error writing {file}: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Err(message::StripError::EmptyMessage) => {
            eprintln!("error: commit message would be empty after stripping co-author lines");
            ExitCode::from(1)
        }
    }
}

fn pick_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .map_err(|e| format!("failed to list branches: {e}"))?;

    if !output.status.success() {
        return Err("not a git repository".to_string());
    }

    let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect();

    if branches.is_empty() {
        return Err("no branches found".to_string());
    }

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt("Select base branch")
        .items(&branches)
        .default(0)
        .interact()
        .map_err(|e| format!("selection cancelled: {e}"))?;

    Ok(branches[selection].clone())
}
