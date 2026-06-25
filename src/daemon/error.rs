use crate::{input::UnixSocketInitError, server::WaylandSocketInitError};

#[cfg(feature = "winit")]
use crate::backend::WinitBackendInitError;

#[derive(Debug, thiserror::Error)]
pub enum DaemonInitError {
    #[error("wayland library not available")]
    NoWaylandLib,
    #[error("wayland socket init error {0}")]
    WaylandSocketError(WaylandSocketInitError),
    #[error("unix socket init error {0}")]
    UnixSocketError(UnixSocketInitError),
    #[cfg(feature = "winit")]
    #[error("backend init error {0}")]
    BackendInitError(WinitBackendInitError),
    #[error("io error {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "winit")]
impl From<WinitBackendInitError> for DaemonInitError {
    fn from(value: WinitBackendInitError) -> Self {
        Self::BackendInitError(value)
    }
}

impl From<UnixSocketInitError> for DaemonInitError {
    fn from(value: UnixSocketInitError) -> Self {
        Self::UnixSocketError(value)
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
        DaemonInitError::WaylandSocketError(value)
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
