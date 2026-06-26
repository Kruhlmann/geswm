use std::sync::Arc;

use crate::{client::ClientState, daemon::Daemon};

pub trait ClientConnectionManager {
    fn handle_client_connections(&mut self);
}

impl<Keyboard, Mouse, Backend, L> ClientConnectionManager for Daemon<Keyboard, Mouse, Backend, L> {
    fn handle_client_connections(&mut self) {
        if let Some(unix_stream) = self.server_state.socket.accept().unwrap() {
            self.display
                .handle()
                .insert_client(unix_stream, Arc::new(ClientState::default()))
                .unwrap();
        }
    }
}
