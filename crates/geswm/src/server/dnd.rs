use std::os::fd::OwnedFd;

use smithay::{
    input::Seat,
    wayland::selection::data_device::{ClientDndGrabHandler, ServerDndGrabHandler},
};

use crate::server::ServerState;

impl ClientDndGrabHandler for ServerState {}

impl ServerDndGrabHandler for ServerState {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}
}
