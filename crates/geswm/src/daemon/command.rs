use std::hint::unreachable_unchecked;

#[derive(Debug)]
pub enum LayoutCommand {
    CycleUp,
    CycleDown,
    SendToTop,
    SendToBottom,
    SendUp,
    SendDown,
}

#[derive(Debug)]
pub enum UserCommand {
    Spawn(Vec<String>),
    Layout(LayoutCommand),
}

impl From<Vec<&str>> for UserCommand {
    fn from(args: Vec<&str>) -> Self {
        UserCommand::Spawn(args.iter().map(|s| s.to_string()).collect())
    }
}

impl UserCommand {
    pub fn execute(&self, socket_name: &str) {
        tracing::info!("Execute");
        match self {
            UserCommand::Spawn(args) => {
                if args.is_empty() {
                    tracing::warn!("No command provided to spawn.");
                    return;
                }
                let command = &args[0];
                let command_args = &args[1..];
                tracing::info!(
                    "Spawning command: {} with args: {:?}",
                    command,
                    command_args
                );
                match std::process::Command::new(command)
                    .args(command_args)
                    .env("WAYLAND_DISPLAY", socket_name)
                    .env("XDG_SESSION_TYPE", "wayland")
                    .env_remove("DISPLAY")
                    .spawn()
                {
                    Ok(child) => {
                        tracing::info!("Spawned process with PID: {}", child.id());
                    }
                    Err(e) => {
                        tracing::error!("Failed to spawn process: {}", e);
                    }
                }
            }
            _ => unreachable!("not implemented"),
        }
    }
}
