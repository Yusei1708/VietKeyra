use clap::Parser;
use vietkeyra_cli::{run, Cli};

fn main() {
    let cli = Cli::parse();
    if let Err(code) = run(cli) {
        std::process::exit(code);
    }
}
