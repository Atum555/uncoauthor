use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "uncoauthor",
    version,
    about = "Remove Co-authored-by trailers from commits in a range"
)]
pub struct Cli {
    /// Base ref (branch, tag, or SHA) — rewrites commits in <base-ref>..HEAD
    pub base_ref: Option<String>,

    /// Generate shell completions and print to stdout
    #[arg(long = "completions", value_name = "SHELL")]
    pub completions: Option<Shell>,

    #[command(subcommand)]
    pub command: Option<InternalCommand>,
}

#[derive(Subcommand)]
pub enum InternalCommand {
    /// Rewrite rebase todo: pick -> reword
    #[command(name = "__sequence-edit", hide = true)]
    SequenceEdit { file: String },

    /// Strip co-authored-by lines from commit message
    #[command(name = "__msg-edit", hide = true)]
    MsgEdit { file: String },
}

pub fn print_completions(shell: Shell) {
    // Build a minimal command just for completion generation to avoid
    // the panic caused by mixing optional positional args with subcommands.
    let mut cmd = clap::Command::new("uncoauthor")
        .arg(
            clap::Arg::new("base_ref")
                .help("Base ref (branch, tag, or SHA)")
                .value_hint(clap::ValueHint::Other),
        )
        .arg(
            clap::Arg::new("completions")
                .long("completions")
                .value_name("SHELL")
                .value_parser(clap::value_parser!(Shell))
                .help("Generate shell completions and print to stdout"),
        );

    clap_complete::generate(shell, &mut cmd, "uncoauthor", &mut std::io::stdout());
}
