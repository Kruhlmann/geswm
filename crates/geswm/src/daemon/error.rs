use crate::{backend::WinitBackendInitError, server::WaylandSocketInitError};

#[derive(Debug, thiserror::Error)]
pub enum DaemonInitError {
    #[error("wayland library not available")]
    NoWaylandLib,
    #[error("socket init error")]
    SocketError(WaylandSocketInitError),
    #[error("backend init error")]
    BackendInitError(WinitBackendInitError),
    #[error("io error {0}")]
    Io(std::io::Error),
}

impl From<WinitBackendInitError> for DaemonInitError {
    fn from(value: WinitBackendInitError) -> Self {
        Self::BackendInitError(value)
    }
}

impl From<wayland_server::backend::InitError> for DaemonInitError {
    fn from(value: wayland_server::backend::InitError) -> Self {
        match value {
            wayland_server::backend::InitError::NoWaylandLib => DaemonInitError::NoWaylandLib,
            wayland_server::backend::InitError::Io(e) => DaemonInitError::Io(e),
        }
    }
}

impl From<WaylandSocketInitError> for DaemonInitError {
    fn from(value: WaylandSocketInitError) -> Self {
        DaemonInitError::SocketError(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonKeyboardInitError {
    #[error("invalid xkb keymap")]
    InvalidKeyMap,
    #[error("io error {0}")]
    Io(std::io::Error),
}

impl From<smithay::input::keyboard::Error> for DaemonKeyboardInitError {
    fn from(value: smithay::input::keyboard::Error) -> Self {
        match value {
            smithay::input::keyboard::Error::BadKeymap => DaemonKeyboardInitError::InvalidKeyMap,
            smithay::input::keyboard::Error::IoError(e) => DaemonKeyboardInitError::Io(e),
        }
    }
}
