use smithay::wayland::shm::{ShmHandler, ShmState};

use crate::server::ServerState;

impl ShmHandler for ServerState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

smithay::delegate_shm!(ServerState);
