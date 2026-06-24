use clap::Subcommand;

use crate::message::SocketMessage;

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Msg {
        #[arg(index = 1)]
        message: SocketMessage,
    },
    Server {},
}
