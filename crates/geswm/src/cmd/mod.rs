pub mod executor;

#[derive(Debug, Clone)]
pub enum LayoutCommand {
    CycleUp,
    CycleDown,
    SendToTop,
    SendToBottom,
    SendUp(u16),
    SendDown(u16),
}

#[derive(Debug, Clone)]
pub enum UserCommand {
    Spawn(Vec<String>),
    Layout(LayoutCommand),
    CloseFocused,
}

impl From<Vec<&str>> for UserCommand {
    fn from(args: Vec<&str>) -> Self {
        UserCommand::Spawn(args.iter().map(|s| s.to_string()).collect())
    }
}

impl std::fmt::Display for UserCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserCommand::Spawn(args) => write!(f, "Spawn('{}')", args.join(" ")),
            UserCommand::Layout(layout) => write!(f, "Layout::{layout:?}"),
            _ => write!(f, "{:?}", self),
        }
    }
}
