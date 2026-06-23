use smithay::wayland::compositor::CompositorClientState;
use wayland_server::backend::{ClientId, DisconnectReason};

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl wayland_server::backend::ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        tracing::info!("client initialized");
    }

    fn disconnected(&self, _client_id: ClientId, reason: DisconnectReason) {
        tracing::warn!("client disconnected: {reason:?}");
    }
}
