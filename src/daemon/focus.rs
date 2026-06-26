use smithay::{input::keyboard::KeyboardHandle, utils::SERIAL_COUNTER};
use wayland_server::protocol::wl_surface::WlSurface;

use crate::{
    daemon::{Daemon, keyboard::NoKeyboard},
    server::ServerState,
};

pub trait FocusHandler {
    fn ensure_a_window_is_focused(&mut self) {
        tracing::warn!("FocusHandler::update_focus called but not implemented");
    }
    fn focus_next(&mut self) {
        tracing::warn!("FocusHandler::focus_next called but not implemented");
    }
    fn focus_prev(&mut self) {
        tracing::warn!("FocusHandler::focus_prev called but not implemented");
    }
    fn clear_focus(&mut self) {
        tracing::warn!("FocusHandler::clear_focus called but not implemented");
    }
    fn move_focused_window_down(&mut self) {
        tracing::warn!("FocusHandler::move_focused_window_down called but not implemented");
    }
    fn move_focused_window_up(&mut self) {
        tracing::warn!("FocusHandler::move_focused_window_up called but not implemented");
    }
}

impl<Mouse, Backend, L> FocusHandler for Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn move_focused_window_down(&mut self) {
        let Some(focused_surface) = self.server_state.focused_window.as_ref() else {
            return;
        };

        let Some(index) = self
            .server_state
            .windows
            .iter()
            .position(|window| &window.surface == focused_surface)
        else {
            return;
        };

        if index == 0 {
            return;
        }

        self.server_state.windows.swap(index, index - 1);
        self.server_state.mark_layout_dirty();
    }

    fn move_focused_window_up(&mut self) {
        let Some(focused_surface) = self.server_state.focused_window.as_ref() else {
            return;
        };

        let Some(index) = self
            .server_state
            .windows
            .iter()
            .position(|window| &window.surface == focused_surface)
        else {
            return;
        };

        if index + 1 >= self.server_state.windows.len() {
            return;
        }

        self.server_state.windows.swap(index, index + 1);
        self.server_state.mark_layout_dirty();
    }

    fn ensure_a_window_is_focused(&mut self) {
        if self.server_state.windows.is_empty() {
            return self.clear_focus();
        }
        if !self.focused_window_still_valid() {
            self.focus_any();
        }
    }

    fn focus_next(&mut self) {
        let Some(focused_surface) = self.server_state.focused_window.as_ref() else {
            return;
        };
        let current_index = self
            .server_state
            .windows
            .iter()
            .position(|window| &window.surface == focused_surface);
        let next_index = current_index.map(|i| (i + 1) % self.server_state.windows.len());
        if let Some(next_surface) = next_index.and_then(|i| self.server_state.windows.get(i)) {
            self.focus_surface(next_surface.surface.clone());
        } else {
            tracing::warn!("No focusable window found when trying to focus next");
        }
    }

    fn focus_prev(&mut self) {
        let Some(focused_surface) = self.server_state.focused_window.as_ref() else {
            return;
        };
        let current_index = self
            .server_state
            .windows
            .iter()
            .position(|window| &window.surface == focused_surface);
        match current_index {
            Some(0) => {
                // If the current index is 0, wrap around to the last window
                let last_index = self.server_state.windows.len() - 1;
                if let Some(previous_surface) = self.server_state.windows.get(last_index) {
                    self.focus_surface(previous_surface.surface.clone());
                } else {
                    tracing::warn!("No focusable window found when trying to focus previous");
                }
            }
            Some(i) => {
                if let Some(previous_surface) = self.server_state.windows.get(i - 1) {
                    self.focus_surface(previous_surface.surface.clone());
                } else {
                    tracing::warn!("No focusable window found when trying to focus previous");
                }
            }
            None => {
                tracing::warn!("Current focused surface not found in windows list");
            }
        }
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

impl<Mouse, Backend, L> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn focused_window_still_valid(&self) -> bool {
        let Some(focused) = self.server_state.focused_window.as_ref() else {
            return false;
        };

        self.server_state
            .windows
            .iter()
            .any(|window| window.surface == *focused && window.is_alive())
    }

    fn focus_any(&mut self) {
        if let Some(window) = self.server_state.windows.last() {
            self.focus_surface(window.surface.clone());
        } else {
            self.clear_focus();
        }
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
}

impl<Mouse, Backend, L> FocusHandler for Daemon<NoKeyboard, Mouse, Backend, L> {}
