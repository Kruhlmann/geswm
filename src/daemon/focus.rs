use smithay::{
    input::keyboard::KeyboardHandle,
    utils::SERIAL_COUNTER,
    wayland::{
        compositor,
        shell::wlr_layer::{KeyboardInteractivity, LayerSurfaceCachedState},
    },
};

use crate::{
    daemon::{Daemon, keyboard::NoKeyboard},
    server::ServerState,
};

pub trait FocusHandler {
    fn ensure_focus(&mut self) {}
}

impl<Mouse, Backend, L> FocusHandler for Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    fn ensure_focus(&mut self) {
        let layer_focus = self
            .server_state
            .layer_shell_state
            .layer_surfaces()
            .rev()
            .find(|surface| {
                surface.alive()
                    && compositor::with_states(surface.wl_surface(), |states| {
                        states
                            .cached_state
                            .get::<LayerSurfaceCachedState>()
                            .current()
                            .keyboard_interactivity
                            != KeyboardInteractivity::None
                    })
            })
            .map(|surface| surface.wl_surface().clone());

        if let Some(surface) = layer_focus {
            if !self.server_state.is_focused(&surface) {
                self.keyboard.set_focus(
                    &mut self.server_state,
                    Some(surface.clone()),
                    SERIAL_COUNTER.next_serial(),
                );
                self.server_state.set_focused_surface(Some(surface));
            }
            return;
        }

        let focused_toplevel_alive =
            self.server_state
                .focused_surface
                .as_ref()
                .is_some_and(|focused| {
                    self.server_state
                        .xdg_shell_state
                        .toplevel_surfaces()
                        .iter()
                        .any(|toplevel| toplevel.wl_surface() == focused)
                });

        if focused_toplevel_alive {
            return;
        }

        if let Some(surface) = self
            .server_state
            .xdg_shell_state
            .toplevel_surfaces()
            .iter()
            .next()
            .map(|toplevel| toplevel.wl_surface().clone())
        {
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
