use smithay::{backend::winit::WinitKeyboardInputEvent, input::keyboard::KeyboardHandle};

use crate::{daemon::Daemon, server::ServerState};

pub type NoKeyboard = ();

pub trait KeyboardHandler {
    fn on_keyboard_event(&mut self, _event: &WinitKeyboardInputEvent);
}

impl<Mouse> KeyboardHandler for Daemon<KeyboardHandle<ServerState>, Mouse> {
    fn on_keyboard_event(&mut self, event: &WinitKeyboardInputEvent) {
        tracing::info!("kbd: {event:?}");
    }
}

impl<Mouse> KeyboardHandler for Daemon<NoKeyboard, Mouse> {
    fn on_keyboard_event(&mut self, event: &WinitKeyboardInputEvent) {
        tracing::warn!("kbd: {event:?}");
    }
}
