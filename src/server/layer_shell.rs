use smithay::wayland::shell::{
    wlr_layer::{Layer, LayerSurface, WlrLayerShellHandler, WlrLayerShellState},
    xdg::PopupSurface,
};

use wayland_server::protocol::wl_output::WlOutput;

use crate::server::ServerState;

impl WlrLayerShellHandler for ServerState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        _output: Option<WlOutput>,
        layer: Layer,
        namespace: String,
    ) {
        tracing::info!(?layer, ?namespace, "new layer-shell surface");

        surface.with_pending_state(|state| {
            state.size = Some((640, 480).into());
        });

        surface.send_configure();
    }

    fn new_popup(&mut self, _parent: LayerSurface, popup: PopupSurface) {
        popup.send_configure().unwrap();
    }

    fn layer_destroyed(&mut self, surface: LayerSurface) {
        if self.is_focused(surface.wl_surface()) {
            self.set_focused_surface(None);
        }

        self.mark_layout_dirty();
    }
}

smithay::delegate_layer_shell!(ServerState);
