use std::path::PathBuf;

use wayland_server::{BindError, ListeningSocket};

const SOCKET_PREFIX: &str = "geswm";
const MAX_SOCKET_INDEX: usize = 256;

#[derive(Debug, thiserror::Error)]
pub enum WaylandSocketInitError {
    #[error("XDG_RUNTIME_DIR is not set; cannot create Wayland socket")]
    RuntimeDirNotSet,
    #[error("no free Wayland socket found ({SOCKET_PREFIX}-1..{MAX_SOCKET_INDEX})")]
    NoAvailableSocket,
}

pub struct WaylandSocket {
    pub socket: ListeningSocket,
    pub name: String,
}

impl WaylandSocket {
    pub fn try_autocreate() -> Result<WaylandSocket, WaylandSocketInitError> {
        for i in 1..MAX_SOCKET_INDEX {
            let name = format!("{SOCKET_PREFIX}-{i}");
            match ListeningSocket::bind(&name) {
                Ok(socket) => {
                    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
                        .ok_or(WaylandSocketInitError::RuntimeDirNotSet)
                        .map(PathBuf::from)?;
                    let path = runtime_dir.join(&name);
                    tracing::info!(?path, "opened socket");
                    return Ok(Self { socket, name });
                }
                Err(BindError::RuntimeDirNotSet) => {
                    return Err(WaylandSocketInitError::RuntimeDirNotSet);
                }
                Err(BindError::PermissionDenied) => tracing::warn!("permission denied"),
                Err(BindError::Io(error)) => tracing::warn!(?error, "io error on socket bind"),
                Err(BindError::AlreadyInUse) => tracing::warn!(?name, "socket in use"),
            };
        }
        Err(WaylandSocketInitError::NoAvailableSocket)
    }

    pub fn display_name(&self) -> &str {
        &self.name
    }
}

impl std::ops::Deref for WaylandSocket {
    type Target = ListeningSocket;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}
