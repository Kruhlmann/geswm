use smithay::wayland::selection::data_device::{DataDeviceHandler, DataDeviceState};

use crate::server::ServerState;

impl DataDeviceHandler for ServerState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

smithay::delegate_data_device!(ServerState);
