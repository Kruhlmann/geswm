use smithay::wayland::selection::SelectionHandler;

use crate::server::ServerState;

impl SelectionHandler for ServerState {
    type SelectionUserData = ();
}
