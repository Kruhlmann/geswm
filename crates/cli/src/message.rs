use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum SocketMessage {
    FocusUp,
    FocusDown,
}
