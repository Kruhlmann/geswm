pub mod executor;

#[derive(Debug, Clone)]
pub enum LayoutCommand {
    CycleUp,
    CycleDown,
    SendToTop,
    SendToBottom,
    SendUp,
    SendDown,
    Shrink,
    Grow,
    IncreaseMasterCount,
    DecreaseMasterCount,
    CycleLayout,
}

#[derive(Debug, Clone)]
pub enum WmSessionCommand {
    Spawn(Vec<String>),
    Layout(LayoutCommand),
    CloseFocused,
    ConfirmCommand(String, Box<Self>),
    GoToWorkSpace(u16),
    MoveFocusedWindowToWorkSpace(u16),
}

impl From<Vec<&str>> for WmSessionCommand {
    fn from(args: Vec<&str>) -> Self {
        WmSessionCommand::Spawn(args.iter().map(|s| s.to_string()).collect())
    }
}

impl From<&str> for WmSessionCommand {
    fn from(command: &str) -> Self {
        WmSessionCommand::Spawn(vec![command.to_string()])
    }
}

impl std::fmt::Display for WmSessionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WmSessionCommand::Spawn(args) => write!(f, "Spawn('{}')", args.join(" ")),
            WmSessionCommand::Layout(layout) => write!(f, "Layout::{layout:?}"),
            _ => write!(f, "{:?}", self),
        }
    }
}
