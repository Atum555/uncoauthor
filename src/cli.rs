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

    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut cmd, "uncoauthor", &mut buf);
    let script = String::from_utf8(buf).expect("clap generates valid UTF-8");

    print!("{}", patch_git_ref_completions(shell, script));
}

fn patch_git_ref_completions(shell: Shell, script: String) -> String {
    match shell {
        Shell::Bash => patch_bash(script),
        Shell::Zsh => patch_zsh(script),
        Shell::Fish => patch_fish(script),
        _ => script,
    }
}

const GIT_REF_CMD: &str =
    "git for-each-ref --format='%(refname:short)' refs/heads/ refs/tags/ 2>/dev/null";

fn patch_bash(script: String) -> String {
    script.replace(
        "COMPREPLY=()\n                    ;;",
        &format!(
            "COMPREPLY=($(compgen -W \"$({GIT_REF_CMD})\" -- \"${{cur}}\"))\n                    ;;"
        ),
    )
}

fn patch_zsh(script: String) -> String {
    let helper = concat!(
        "__uncoauthor_git_refs() {\n",
        "    local -a refs\n",
        "    refs=(${(f)\"$(git for-each-ref --format='%(refname:short)' refs/heads/ refs/tags/ 2>/dev/null)\"})\n",
        "    compadd -a refs\n",
        "}\n",
    );

    let script = script.replace(
        "'::base_ref -- Base ref (branch, tag, or SHA):' \\",
        "'::base_ref -- Base ref (branch, tag, or SHA):__uncoauthor_git_refs' \\",
    );

    script.replacen(
        "#compdef uncoauthor\n",
        &format!("#compdef uncoauthor\n\n{helper}\n"),
        1,
    )
}

fn patch_fish(script: String) -> String {
    format!(
        "{script}\n# Complete base_ref with git branches and tags\n\
         complete -c uncoauthor -f -n '__fish_use_subcommand' \
         -a '(git for-each-ref --format=\"%(refname:short)\" refs/heads/ refs/tags/ 2>/dev/null)' \
         -d 'Git ref'\n"
    )
}
