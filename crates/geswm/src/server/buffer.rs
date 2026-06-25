use smithay::wayland::buffer::BufferHandler;

use crate::server::ServerState;

impl BufferHandler for ServerState {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {}
}
