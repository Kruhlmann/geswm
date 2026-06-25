use crate::cmd::WmSessionCommand;

pub trait UserCommandExecutor {
    type Error;

    fn execute(&self, _command: &WmSessionCommand) -> Result<(), Self::Error>;
}

pub struct DaemonCommandExecutor {
    socket_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonCommandExecutionError {
    #[error("spawn command failed {0}")]
    SpawnError(std::io::Error),
    #[error("spawn command had no command")]
    NoCommand,
}

impl DaemonCommandExecutor {
    pub fn new(socket_name: String) -> DaemonCommandExecutor {
        Self { socket_name }
    }

    pub fn exec_spawn(
        &self,
        command_segments: &Vec<String>,
    ) -> Result<(), DaemonCommandExecutionError> {
        let (command, args) = match command_segments.as_slice() {
            [] => return Err(DaemonCommandExecutionError::NoCommand),
            [c, a @ ..] => (c, a),
        };
        std::process::Command::new(command)
            .args(args)
            .env("WAYLAND_DISPLAY", &self.socket_name)
            .env("XDG_SESSION_TYPE", "wayland")
            .env_remove("DISPLAY")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(DaemonCommandExecutionError::SpawnError)
            .inspect(|child| tracing::info!(?command, id = child.id(), "spawned process"))?;
        Ok(())
    }

    pub fn show_prompt(_prompt: &str) -> bool {
        true
    }
}

impl UserCommandExecutor for DaemonCommandExecutor {
    type Error = DaemonCommandExecutionError;

    fn execute(&self, command: &WmSessionCommand) -> Result<(), Self::Error> {
        match command {
            WmSessionCommand::Spawn(args) => self.exec_spawn(args),
            WmSessionCommand::Layout(_layout_command) => todo!(),
            WmSessionCommand::CloseFocused => todo!(),
            WmSessionCommand::ConfirmCommand(prompt, next_command) => {
                match Self::show_prompt(prompt) {
                    true => self.execute(next_command),
                    false => Ok(()),
                }
            }
            WmSessionCommand::GoToWorkSpace(_) => todo!(),
            WmSessionCommand::MoveFocusedWindowToWorkSpace(_) => todo!(),
        }
    }
}
