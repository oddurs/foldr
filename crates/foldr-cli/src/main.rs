use clap::Parser;

/// Inspect and configure folders on macOS and Linux.
#[derive(Parser)]
#[command(name = "foldr", version, arg_required_else_help = true)]
#[command(after_help = "Early development: folder commands are tracked in ROADMAP.md.")]
struct Cli {}

fn main() {
    Cli::parse();
}
