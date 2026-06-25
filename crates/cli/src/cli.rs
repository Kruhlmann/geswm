
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "geswmcli")]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    #[command(about = "Start the server and listen for incoming connections")]
    Server,
    #[command(about = "Send a message to the server socket")]
    Msg {
        #[arg(index = 1)]
        message: SocketMessage,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SocketMessage {
    FocusUp,
    FocusDown,
}
