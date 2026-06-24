use clap::Parser;

use crate::command::CliCommand;

#[derive(Debug, Parser)]
#[command(name = "geswmcli")]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}
