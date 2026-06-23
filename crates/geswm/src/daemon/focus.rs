use smithay::{input::keyboard::KeyboardHandle, utils::SERIAL_COUNTER};

use crate::{daemon::{Daemon, keyboard::NoKeyboard}, server::ServerState};

pub trait FocusHandler {
    fn ensure_focus(&mut self) {}
}

impl<Mouse, Backend, L> FocusHandler for Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn ensure_focus(&mut self) {
        if self.server_state.focused_surface.is_some() {
            return;
        }

        let surface = self
            .server_state
            .xdg_shell_state
            .toplevel_surfaces()
            .iter()
            .next()
            .map(|toplevel| toplevel.wl_surface().clone());

        if let Some(surface) = surface {
            self.keyboard.set_focus(
                &mut self.server_state,
                Some(surface.clone()),
                SERIAL_COUNTER.next_serial(),
            );
            self.server_state.set_focused_surface(Some(surface));
        }
    }
}

impl<Mouse, Backend, L> FocusHandler for Daemon<NoKeyboard, Mouse, Backend, L> {}
