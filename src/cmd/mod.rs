#[derive(Debug, Clone)]
pub enum LayoutCmd {
    FocusNext,
    FocusPrev,
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
pub enum Cmd {
    Spawn(Vec<String>),
    Layout(LayoutCmd),
    CloseFocused,
    ConfirmCommand(String, Box<Self>),
    GoToWorkSpace(u16),
    MoveFocusedWindowToWorkSpace(u16),
    Exit(i32),
}

impl From<Vec<&str>> for Cmd {
    fn from(args: Vec<&str>) -> Self {
        Cmd::Spawn(args.iter().map(|s| s.to_string()).collect())
    }
}

impl From<&str> for Cmd {
    fn from(command: &str) -> Self {
        Cmd::Spawn(vec![command.to_string()])
    }
}

impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cmd::Spawn(args) => write!(f, "Spawn('{}')", args.join(" ")),
            Cmd::Layout(layout) => write!(f, "Layout::{layout:?}"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandExecutionError {
    #[error("spawn command failed {0}")]
    SpawnError(std::io::Error),
    #[error("spawn command had no command")]
    NoCommand,
}

impl Cmd {
    pub fn exec_spawn(
        &self,
        command_segments: &Vec<String>,
        socket_name: &str,
    ) -> Result<(), CommandExecutionError> {
        let (command, args) = match command_segments.as_slice() {
            [] => return Err(CommandExecutionError::NoCommand),
            [c, a @ ..] => (c, a),
        };
        std::process::Command::new(command)
            .args(args)
            .env("WAYLAND_DISPLAY", socket_name)
            .env("XDG_SESSION_TYPE", "wayland")
            .env_remove("DISPLAY")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(CommandExecutionError::SpawnError)
            .inspect_err(|error| tracing::error!(?command, ?error, "failed to spawn process"))
            .inspect(|child| tracing::info!(?command, id = child.id(), "spawned process"))?;
        Ok(())
    }

    pub fn show_prompt(_prompt: &str) -> bool {
        true
    }
}
