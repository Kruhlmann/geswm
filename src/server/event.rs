use crate::{
    backend::{BackendEvent, GesWmBackend, InputEvent},
    cmd::WmSessionCommand,
    daemon::{Daemon, focus::FocusHandler, keyboard::KeyboardHandler, mouse::MouseHandler},
    layout::Layout,
    server::ServerState,
};

pub trait BackendEventHandler {
    fn handle_backend_event(&mut self, _event: &BackendEvent) -> Option<WmSessionCommand> {
        tracing::warn!("BackendEventHandler::handle_backend_event called but not implemented");
        None
    }
}

impl<Keyboard, Mouse, Backend: GesWmBackend<ServerState>, L> BackendEventHandler
    for Daemon<Keyboard, Mouse, Backend, L>
where
    Daemon<Keyboard, Mouse, Backend, L>: KeyboardHandler + MouseHandler + FocusHandler,
    L: Layout,
{
    fn handle_backend_event(&mut self, event: &BackendEvent) -> Option<WmSessionCommand> {
        match event {
            BackendEvent::Resize { size, scale } => {
                tracing::info!("resized event: {size:?} scale: {scale:?}");
                None
            }
            BackendEvent::FocusGained => None,
            BackendEvent::FocusLost => None,
            BackendEvent::Input(input_event) => match input_event {
                InputEvent::Keyboard { time, key, state } => {
                    self.on_keyboard_event(*time, key, state)
                }
                InputEvent::PointerAxis { time } => self.on_mouse_wheel_event(time),
                InputEvent::PointerButton {
                    time,
                    button_code,
                    state,
                } => self.on_mouse_input_event(time, button_code, state),
                InputEvent::PointerMotionAbsolute { time, x, y } => {
                    self.on_mouse_moved_event(time, x, y)
                }
                InputEvent::Unimplemented => None,
            },
            BackendEvent::Redraw => {
                tracing::info!("redraw event");
                None
            }
            BackendEvent::CloseRequested => Some(WmSessionCommand::Exit(0)),
        }
    }
}
