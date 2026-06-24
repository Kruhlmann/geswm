use clap::Parser;
use cli::cli::Cli;

fn main() {
    let cli = Cli::parse();
    eprintln!("{cli:?}")
}
