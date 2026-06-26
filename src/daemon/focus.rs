use smithay::{
    input::keyboard::KeyboardHandle,
    utils::SERIAL_COUNTER,
};
use wayland_server::protocol::wl_surface::WlSurface;

use crate::{
    daemon::{Daemon, keyboard::NoKeyboard},
    server::ServerState,
};

pub trait FocusHandler {
    fn update_focus(&mut self) {}
}

impl<Mouse, Backend, L> FocusHandler for Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn update_focus(&mut self) {
        if self.focused_window_still_valid() {
            return;
        }
        match self.next_focusable_window() {
            Some(surface) => self.focus_surface(surface),
            None => self.clear_focus(),
        };
    }
}

impl<Mouse, Backend, L> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn focused_window_still_valid(&self) -> bool {
        let Some(focused) = self.server_state.focused_window.as_ref() else {
            return false;
        };

        self.server_state
            .windows
            .iter()
            .any(|window| window.surface == *focused && window.toplevel.alive())
    }

    fn next_focusable_window(&self) -> Option<WlSurface> {
        self.server_state
            .windows
            .iter()
            .rev()
            .find(|window| window.toplevel.alive())
            .map(|window| window.surface.clone())
    }

    fn focus_surface(&mut self, surface: WlSurface) {
        self.keyboard.set_focus(
            &mut self.server_state,
            Some(surface.clone()),
            SERIAL_COUNTER.next_serial(),
        );

        self.server_state.set_focused_surface(Some(surface));
        self.server_state.mark_layout_dirty();
    }

    fn clear_focus(&mut self) {
        if self.server_state.focused_window.is_none() {
            return;
        }

        self.keyboard
            .set_focus(&mut self.server_state, None, SERIAL_COUNTER.next_serial());

        self.server_state.set_focused_surface(None);
        self.server_state.mark_layout_dirty();
    }
}

impl<Mouse, Backend, L> FocusHandler for Daemon<NoKeyboard, Mouse, Backend, L> {}
