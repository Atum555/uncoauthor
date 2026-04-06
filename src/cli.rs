use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "uncoauthor",
    version,
    about = "Remove Co-authored-by trailers from commits in a range"
)]
pub struct Cli {
    /// Base ref (branch, tag, or SHA) — rewrites commits in <base-ref>..HEAD
    pub base_ref: Option<String>,

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
