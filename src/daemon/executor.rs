use crate::{
    backend::GesWmBackend,
    cmd::{LayoutCommand, WmSessionCommand},
    daemon::{Daemon, focus::FocusHandler, keyboard::KeyboardHandler, mouse::MouseHandler},
    layout::Layout,
    server::ServerState,
};

pub trait CommandExecutor<Cmd> {
    fn execute(&mut self, command: &Cmd);
}

impl<Keyboard, Mouse, Backend, L> CommandExecutor<WmSessionCommand>
    for Daemon<Keyboard, Mouse, Backend, L>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, L>:
        KeyboardHandler + MouseHandler + FocusHandler + CommandExecutor<LayoutCommand>,
    L: Layout,
{
    fn execute(&mut self, command: &WmSessionCommand) {
        match command {
            WmSessionCommand::Spawn(cmd) => {
                command
                    .exec_spawn(cmd, self.server_state.socket_name())
                    .inspect_err(|error| tracing::error!(?error, "spawn failed"))
                    .ok();
            }
            WmSessionCommand::Layout(layout_command) => {
                <Self as CommandExecutor<LayoutCommand>>::execute(self, layout_command)
            }
            WmSessionCommand::CloseFocused => self.close_focused_window(),
            WmSessionCommand::ConfirmCommand(prompt, next_command) => {
                if WmSessionCommand::show_prompt(prompt) {
                    <Self as CommandExecutor<WmSessionCommand>>::execute(
                        self,
                        next_command.as_ref(),
                    );
                }
            }
            WmSessionCommand::GoToWorkSpace(_) => todo!(),
            WmSessionCommand::MoveFocusedWindowToWorkSpace(_) => todo!(),
            WmSessionCommand::Exit(..) => {}
        };
    }
}

impl<Keyboard, Mouse, Backend, L> CommandExecutor<LayoutCommand>
    for Daemon<Keyboard, Mouse, Backend, L>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, L>: KeyboardHandler + MouseHandler + FocusHandler,
    L: Layout,
{
    fn execute(&mut self, command: &LayoutCommand) {
        match command {
            LayoutCommand::FocusNext => self.focus_next(),
            LayoutCommand::FocusPrev => self.focus_prev(),
            LayoutCommand::SendDown => self.move_focused_window_down(),
            LayoutCommand::SendUp => self.move_focused_window_up(),
            _ => todo!(),
        };
    }
}
