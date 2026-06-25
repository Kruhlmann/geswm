use clap::Parser;

use cli::{
    cli::{Cli, CliCommand},
    log,
};

fn main() {
    log::setup_logging();
    let res = match Cli::parse().command {
        CliCommand::Server => cli::run_server(),
        CliCommand::Msg { .. } => todo!(),
    };
    if let Err(e) = res {
        tracing::error!(?e, "Error executing command");
        std::process::exit(1);
    }
}
