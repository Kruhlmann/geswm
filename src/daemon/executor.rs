use crate::{
    backend::GesWmBackend,
    cmd::{Cmd, LayoutCmd},
    daemon::{
        Daemon, focus::FocusHandler, keyboard::KeyboardHandler, mouse::MouseHandler,
        window::WindowManager,
    },
    layout::LayoutSet,
    server::ServerState,
};

pub trait CommandExecutor<Cmd> {
    fn execute(&mut self, command: &Cmd);
}

impl<Keyboard, Mouse, Backend, L> CommandExecutor<Cmd> for Daemon<Keyboard, Mouse, Backend, L>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, L>:
        MouseHandler + FocusHandler + CommandExecutor<LayoutCmd> + WindowManager,
{
    fn execute(&mut self, command: &Cmd) {
        match command {
            Cmd::Spawn(cmd) => {
                command
                    .exec_spawn(cmd, self.server_state.socket_name())
                    .inspect_err(|error| tracing::error!(?error, "spawn failed"))
                    .ok();
            }
            Cmd::Layout(layout_command) => {
                <Self as CommandExecutor<LayoutCmd>>::execute(self, layout_command)
            }
            Cmd::CloseFocused => self.close_focused_window(),
            Cmd::ConfirmCommand(prompt, next) => {
                if Cmd::show_prompt(prompt) {
                    <Self as CommandExecutor<Cmd>>::execute(self, next.as_ref());
                }
            }
            Cmd::GoToWorkSpace(_) => todo!(),
            Cmd::MoveFocusedWindowToWorkSpace(_) => todo!(),
            Cmd::Exit(code) => {
                tracing::info!(?code, "external exit request received");
                std::process::exit(*code);
            }
        };
    }
}

impl<Keyboard, Mouse, Backend> CommandExecutor<LayoutCmd>
    for Daemon<Keyboard, Mouse, Backend, LayoutSet>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, LayoutSet>:
        KeyboardHandler + MouseHandler + FocusHandler + WindowManager,
{
    fn execute(&mut self, command: &LayoutCmd) {
        match command {
            LayoutCmd::FocusNext => self.focus_next(),
            LayoutCmd::FocusPrev => self.focus_prev(),
            LayoutCmd::SendDown => self.move_focused_window_down(),
            LayoutCmd::SendUp => self.move_focused_window_up(),
            LayoutCmd::CycleLayout => self.cycle_layout(),
            LayoutCmd::Grow => self.get_active_layout().grow(),
            LayoutCmd::Shrink => self.get_active_layout().shrink(),
            _ => todo!(),
        };
    }
}
