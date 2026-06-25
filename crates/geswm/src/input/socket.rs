use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
};

const MAX_SOCKET_INDEX: usize = 256;

#[derive(Debug)]
pub struct UnixSocket {
    listener: UnixListener,
    name: String,
    path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum UnixSocketInitError {
    #[error("XDG_RUNTIME_DIR is not set")]
    RuntimeDirNotSet,
    #[error("no available socket name")]
    NoAvailableSocket,
    #[error("permission denied")]
    PermissionDenied,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl UnixSocket {
    pub fn try_autocreate(socket_prefix: &str) -> Result<Self, UnixSocketInitError> {
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
            .ok_or(UnixSocketInitError::RuntimeDirNotSet)
            .map(PathBuf::from)?;

        for i in 0..MAX_SOCKET_INDEX {
            let name = format!("{socket_prefix}-{i}");
            let path = runtime_dir.join(&name);

            match bind_listener(&path) {
                Ok(listener) => {
                    tracing::info!(?path, "opened socket");
                    return Ok(Self {
                        listener,
                        name,
                        path,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
                    tracing::warn!(?path, "socket already in use");
                }
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    tracing::warn!(?path, "permission denied");
                    return Err(UnixSocketInitError::PermissionDenied);
                }
                Err(error) => {
                    tracing::warn!(?path, ?error, "failed to bind unix socket");
                }
            }
        }

        Err(UnixSocketInitError::NoAvailableSocket)
    }

    pub fn display_name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn listener(&self) -> &UnixListener {
        &self.listener
    }
}

fn bind_listener(path: &Path) -> Result<UnixListener, std::io::Error> {
    match UnixListener::bind(path) {
        Ok(listener) => Ok(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
            if stale_socket(path)? {
                tracing::info!(?path, "removing stale socket");
                std::fs::remove_file(path)?;
                UnixListener::bind(path)
            } else {
                Err(error)
            }
        }
        Err(error) => Err(error),
    }
}

fn stale_socket(path: &Path) -> Result<bool, std::io::Error> {
    match UnixStream::connect(path) {
        Ok(stream) => {
            drop(stream);
            Ok(false)
        }
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
            ) =>
        {
            Ok(true)
        }
        Err(error) => Err(error),
    }
}

impl std::ops::Drop for UnixSocket {
    fn drop(&mut self) {
        match std::fs::remove_file(&self.path) {
            Ok(..) => tracing::info!(path = ?self.path, "clean up socket"),
            Err(error) => tracing::error!(?error, path = ?self.path, "clean up socket"),
        };
    }
}
