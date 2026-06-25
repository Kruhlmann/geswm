use smithay::wayland::output::{OutputHandler, OutputManagerState};

use crate::server::ServerState;

impl OutputHandler for ServerState {
    fn output_bound(
        &mut self,
        _output: smithay::output::Output,
        _wl_output: wayland_server::protocol::wl_output::WlOutput,
    ) {
    }
}

impl ServerState {
    pub fn output_manager_state(&mut self) -> &mut OutputManagerState {
        &mut self.output_manager_state
    }
}

smithay::delegate_output!(ServerState);
