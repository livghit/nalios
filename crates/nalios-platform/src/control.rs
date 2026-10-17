//! Private shell-control socket.
//!
//! One socket per user session at `$XDG_RUNTIME_DIR/nalios-dev/control.sock`,
//! mode 0600 inside a 0700 directory. Requests are newline-delimited JSON and
//! bounded before parsing. Linux peer credentials are checked so a socket with
//! wrong ownership cannot be used.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use nalios_app::control::{ControlRequest, ControlResponse};

/// Maximum accepted control message size in bytes.
pub const MAX_CONTROL_BYTES: usize = 64 * 1024;

const NAMESPACE: &str = "nalios-dev";
const SOCKET_NAME: &str = "control.sock";

/// A control transport failure.
#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    /// `XDG_RUNTIME_DIR` is missing or not absolute.
    #[error("XDG_RUNTIME_DIR is not set to an absolute path")]
    NoRuntimeDir,
    /// The runtime namespace path is unsafe (symlink or not a directory).
    #[error("control runtime directory is not a real directory")]
    UnsafeDirectory,
    /// Another shell is already listening.
    #[error("another NaliOS shell is already listening on {0}")]
    AlreadyRunning(PathBuf),
    /// The socket could not be created.
    #[error("could not create the control socket: {0}")]
    Bind(#[source] std::io::Error),
    /// The socket could not be reached.
    #[error("could not connect to the NaliOS shell: {0}")]
    Connect(#[source] std::io::Error),
    /// Communication failed after connecting.
    #[error("control communication failed: {0}")]
    Io(#[source] std::io::Error),
    /// The peer UID does not match the current user.
    #[error("control peer rejected: different user")]
    PeerRejected,
    /// The message was not valid JSON.
    #[error("invalid control message: {0}")]
    Malformed(#[source] serde_json::Error),
    /// The message exceeded the size limit.
    #[error("control message exceeded the {MAX_CONTROL_BYTES}-byte limit")]
    TooLarge,
    /// The request protocol version is unsupported.
    #[error(transparent)]
    Version(#[from] nalios_app::control::ControlVersionError),
}

/// The control socket path for the current user session.
pub fn socket_path() -> Result<PathBuf, ControlError> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").ok_or(ControlError::NoRuntimeDir)?;
    let runtime = PathBuf::from(runtime);
    if !runtime.is_absolute() {
        return Err(ControlError::NoRuntimeDir);
    }
    Ok(runtime.join(NAMESPACE).join(SOCKET_NAME))
}

/// A bound control socket owned by the running shell.
#[derive(Debug)]
pub struct ControlServer {
    listener: UnixListener,
    path: PathBuf,
}

impl ControlServer {
    /// Bind the control socket using the current session environment.
    pub fn bind() -> Result<Self, ControlError> {
        Self::bind_at(&socket_path()?)
    }

    /// Bind the control socket at an explicit path (used by tests).
    pub fn bind_at(socket: &Path) -> Result<Self, ControlError> {
        let directory = socket.parent().ok_or(ControlError::UnsafeDirectory)?;
        prepare_directory(directory)?;

        if socket.exists() {
            match UnixStream::connect(socket) {
                Ok(_) => return Err(ControlError::AlreadyRunning(socket.to_path_buf())),
                Err(_) => {
                    let _ = std::fs::remove_file(socket);
                }
            }
        }

        let listener = UnixListener::bind(socket).map_err(ControlError::Bind)?;
        std::fs::set_permissions(socket, std::fs::Permissions::from_mode(0o600))
            .map_err(ControlError::Bind)?;

        Ok(Self {
            listener,
            path: socket.to_path_buf(),
        })
    }

    /// The bound socket path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Accept one validated connection.
    pub fn accept(&self) -> Result<ControlConnection, ControlError> {
        let (stream, _) = self.listener.accept().map_err(ControlError::Io)?;
        check_peer(&stream)?;
        Ok(ControlConnection { stream })
    }
}

impl Drop for ControlServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// One accepted control connection.
#[derive(Debug)]
pub struct ControlConnection {
    stream: UnixStream,
}

impl ControlConnection {
    /// Read and validate one request.
    pub fn receive(&mut self) -> Result<ControlRequest, ControlError> {
        let mut reader = BufReader::new(&mut self.stream);
        let line = read_bounded_line(&mut reader)?;
        let request: ControlRequest =
            serde_json::from_str(&line).map_err(ControlError::Malformed)?;
        request.validate()?;
        Ok(request)
    }

    /// Send a response.
    pub fn respond(&mut self, response: &ControlResponse) -> Result<(), ControlError> {
        let mut text = serde_json::to_string(response).map_err(ControlError::Malformed)?;
        text.push('\n');
        self.stream
            .write_all(text.as_bytes())
            .map_err(ControlError::Io)?;
        self.stream.flush().map_err(ControlError::Io)
    }
}

/// Send one control request to a running shell.
pub fn send(request: &ControlRequest) -> Result<ControlResponse, ControlError> {
    send_to(&socket_path()?, request)
}

/// Send one control request to an explicit socket path (used by tests).
pub fn send_to(socket: &Path, request: &ControlRequest) -> Result<ControlResponse, ControlError> {
    let mut stream = UnixStream::connect(socket).map_err(ControlError::Connect)?;

    let mut text = serde_json::to_string(request).map_err(ControlError::Malformed)?;
    text.push('\n');
    stream.write_all(text.as_bytes()).map_err(ControlError::Io)?;
    stream.flush().map_err(ControlError::Io)?;

    let mut reader = BufReader::new(&mut stream);
    let line = read_bounded_line(&mut reader)?;
    serde_json::from_str(&line).map_err(ControlError::Malformed)
}

fn prepare_directory(directory: &Path) -> Result<(), ControlError> {
    match std::fs::symlink_metadata(directory) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(ControlError::UnsafeDirectory);
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(directory).map_err(ControlError::Bind)?;
        }
        Err(error) => return Err(ControlError::Bind(error)),
    }

    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
        .map_err(ControlError::Bind)
}

fn check_peer(stream: &UnixStream) -> Result<(), ControlError> {
    let peer = rustix::net::sockopt::socket_peercred(stream)
        .map_err(|error| ControlError::Io(std::io::Error::from(error)))?;
    if peer.uid != rustix::process::geteuid() {
        return Err(ControlError::PeerRejected);
    }
    Ok(())
}

fn read_bounded_line(reader: &mut impl BufRead) -> Result<String, ControlError> {
    let mut bytes = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        let read = reader.read(&mut byte).map_err(ControlError::Io)?;
        if read == 0 {
            return Err(ControlError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "control peer closed the connection",
            )));
        }
        if byte[0] == b'\n' {
            break;
        }
        if bytes.len() >= MAX_CONTROL_BYTES {
            return Err(ControlError::TooLarge);
        }
        bytes.push(byte[0]);
    }

    String::from_utf8(bytes).map_err(|_| {
        ControlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "control message was not valid UTF-8",
        ))
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::thread;

    use nalios_app::control::ControlAction;
    use tempfile::TempDir;

    use super::*;

    fn temp_socket() -> (TempDir, PathBuf) {
        let directory = TempDir::new().expect("temp dir");
        let socket = directory.path().join("control.sock");
        (directory, socket)
    }

    #[test]
    fn round_trip_delivers_the_action() {
        let (_directory, socket) = temp_socket();
        let server = ControlServer::bind_at(&socket).expect("bind");

        let handle = thread::spawn(move || {
            let mut connection = server.accept().expect("accept");
            let request = connection.receive().expect("receive");
            connection.respond(&ControlResponse::Ok).expect("respond");
            request.action
        });

        let response =
            send_to(&socket, &ControlRequest::new(ControlAction::ToggleLauncher)).expect("send");

        assert_eq!(response, ControlResponse::Ok);
        assert_eq!(handle.join().expect("server"), ControlAction::ToggleLauncher);
    }

    #[test]
    fn second_bind_reports_a_running_shell() {
        let (_directory, socket) = temp_socket();
        let _server = ControlServer::bind_at(&socket).expect("bind");

        let error = ControlServer::bind_at(&socket).expect_err("already running");

        assert!(matches!(error, ControlError::AlreadyRunning(_)));
    }

    #[test]
    fn stale_socket_is_replaced() {
        let (_directory, socket) = temp_socket();
        {
            let _server = ControlServer::bind_at(&socket).expect("bind");
        }

        let _server = ControlServer::bind_at(&socket).expect("rebind");
    }

    #[test]
    fn malformed_and_oversized_messages_are_rejected() {
        let (_directory, socket) = temp_socket();
        let server = ControlServer::bind_at(&socket).expect("bind");

        let handle = thread::spawn(move || {
            {
                let mut connection = server.accept().expect("accept");
                let error = connection.receive().expect_err("malformed");
                assert!(matches!(error, ControlError::Malformed(_)));
            }
            {
                let mut connection = server.accept().expect("accept");
                let error = connection.receive().expect_err("too large");
                assert!(matches!(error, ControlError::TooLarge));
            }
        });

        {
            let mut stream = UnixStream::connect(&socket).expect("connect");
            stream.write_all(b"not json\n").expect("write");
        }
        {
            let mut stream = UnixStream::connect(&socket).expect("connect");
            let huge = "x".repeat(MAX_CONTROL_BYTES + 1);
            let _ = stream.write_all(huge.as_bytes());
            let _ = stream.write_all(b"\n");
        }

        handle.join().expect("server");
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let (_directory, socket) = temp_socket();
        let server = ControlServer::bind_at(&socket).expect("bind");

        let handle = thread::spawn(move || {
            let mut connection = server.accept().expect("accept");
            let error = connection.receive().expect_err("version");
            assert!(matches!(error, ControlError::Version(_)));
        });

        let mut stream = UnixStream::connect(&socket).expect("connect");
        stream
            .write_all(b"{\"schema_version\":99,\"action\":\"ping\"}\n")
            .expect("write");
        stream.flush().expect("flush");

        handle.join().expect("server");
    }

    #[test]
    fn missing_socket_reports_a_connect_error() {
        let (_directory, socket) = temp_socket();

        let error = send_to(&socket, &ControlRequest::new(ControlAction::Ping))
            .expect_err("connect error");

        assert!(matches!(error, ControlError::Connect(_)));
    }
}
