use crate::{
    backend::{BackendPumpStatus, GesWmBackend},
    cmd::Cmd,
    daemon::{
        Daemon, executor::CommandExecutor, focus::FocusHandler, keyboard::KeyboardHandler,
        mouse::MouseHandler,
    },
    server::{ServerState, event::BackendEventHandler},
};

pub trait EventProcessor {
    fn handle_new_events(&mut self);
}

impl<Keyboard, Mouse, Backend, L> EventProcessor for Daemon<Keyboard, Mouse, Backend, L>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, L>:
        KeyboardHandler + MouseHandler + FocusHandler + CommandExecutor<Cmd>,
{
    fn handle_new_events(&mut self) {
        let mut event_queue = Vec::new();
        match self
            .backend
            .dispatch_new_events(|event| event_queue.push(event))
        {
            BackendPumpStatus::Continue => {}
            BackendPumpStatus::Exit(exit_code) => {
                tracing::info!(?exit_code, "event loop exited");
                std::process::exit(exit_code);
            }
        };

        let mut commands: Vec<Option<Cmd>> = Vec::new();
        for event in &event_queue {
            commands.push(self.handle_backend_event(event));
        }

        for command in commands.into_iter().flatten() {
            self.execute(&command);
        }
    }
}
