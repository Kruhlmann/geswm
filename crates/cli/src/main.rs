use clap::Parser;

use cli::{
    cli::{Cli, CliCommand},
    log,
};

fn main() {
    log::setup_logging();
    let res = match Cli::parse().command {
        CliCommand::Server => cli::run_server(),
        CliCommand::Msg { message } => todo!(),
    };
}
