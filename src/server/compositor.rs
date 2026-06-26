use smithay::wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState};
use wayland_server::{protocol::wl_surface::WlSurface, Client};

use crate::{client::ClientState, server::ServerState};

impl CompositorHandler for ServerState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("client data missing")
            .compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);
    }
}

smithay::delegate_compositor!(ServerState);
