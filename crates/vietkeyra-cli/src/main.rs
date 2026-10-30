use clap::Parser;
use vietkeyra_cli::{run, Cli};

// VietKeyra CLI: supports status, toggle, set-mode, waybar, monitor subcommands.

fn main() {
    let cli = Cli::parse();
    if let Err(code) = run(cli) {
        std::process::exit(code);
    }
}
