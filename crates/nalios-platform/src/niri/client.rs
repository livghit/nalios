//! Synchronous client for the documented Niri IPC socket.
//!
//! The protocol is newline-delimited JSON. Replies and events are bounded
//! before parsing so a broken or hostile peer cannot make the process allocate
//! without limit.

use std::collections::BTreeMap;
use std::io::{BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use nalios_app::niri::NiriEvent;
use nalios_core::niri::{Output, OutputName, Snapshot, Window, Workspace};

use super::wire::{self, WireError};

/// Maximum accepted reply or event size in bytes.
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// A synchronous connection to the Niri IPC socket for snapshot requests.
#[derive(Debug)]
pub struct NiriClient {
    stream: BufReader<UnixStream>,
}

impl NiriClient {
    /// Connect to the Niri socket at `path`.
    pub fn connect(path: &Path) -> Result<Self, ClientError> {
        let stream = UnixStream::connect(path).map_err(ClientError::Connect)?;
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    /// Read the workspace snapshot.
    pub fn workspaces(&mut self) -> Result<Vec<Workspace>, ClientError> {
        let reply = self.request("Workspaces")?;
        Ok(wire::parse_workspaces(&reply)?)
    }

    /// Read the window snapshot.
    pub fn windows(&mut self) -> Result<Vec<Window>, ClientError> {
        let reply = self.request("Windows")?;
        Ok(wire::parse_windows(&reply)?)
    }

    /// Read the output snapshot.
    pub fn outputs(&mut self) -> Result<BTreeMap<OutputName, Output>, ClientError> {
        let reply = self.request("Outputs")?;
        Ok(wire::parse_outputs(&reply)?)
    }

    /// Read the focused window, if any.
    pub fn focused_window(&mut self) -> Result<Option<Window>, ClientError> {
        let reply = self.request("FocusedWindow")?;
        Ok(wire::parse_focused_window(&reply)?)
    }

    /// Read a read-only snapshot of the supported state.
    pub fn snapshot(&mut self) -> Result<Snapshot, ClientError> {
        Ok(Snapshot {
            workspaces: self.workspaces()?,
            windows: self.windows()?,
            outputs: self.outputs()?,
            focused_window: self.focused_window()?,
        })
    }

    fn request(&mut self, name: &str) -> Result<String, ClientError> {
        let mut line = String::with_capacity(name.len() + 3);
        line.push('"');
        line.push_str(name);
        line.push_str("\"\n");

        self.stream
            .get_mut()
            .write_all(line.as_bytes())
            .map_err(ClientError::Io)?;
        self.stream.get_mut().flush().map_err(ClientError::Io)?;

        read_frame(&mut self.stream)
    }
}

/// A blocking subscription to the Niri event stream.
///
/// Reply acknowledgements and events this build does not model are skipped.
#[derive(Debug)]
pub struct NiriEventStream {
    stream: BufReader<UnixStream>,
}

impl NiriEventStream {
    /// Subscribe to the event stream at `path`.
    pub fn connect(path: &Path) -> Result<Self, ClientError> {
        let stream = UnixStream::connect(path).map_err(ClientError::Connect)?;
        let mut stream = BufReader::new(stream);

        stream
            .get_mut()
            .write_all(b"\"EventStream\"\n")
            .map_err(ClientError::Io)?;
        stream.get_mut().flush().map_err(ClientError::Io)?;

        Ok(Self { stream })
    }

    /// Block until the next modeled event arrives.
    pub fn next_event(&mut self) -> Result<NiriEvent, ClientError> {
        loop {
            let frame = read_frame(&mut self.stream)?;
            if let Some(event) = wire::parse_event(&frame)? {
                return Ok(event);
            }
        }
    }
}

fn read_frame(stream: &mut BufReader<UnixStream>) -> Result<String, ClientError> {
    let mut bytes = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        let read = stream.read(&mut byte).map_err(ClientError::Io)?;
        if read == 0 {
            return Err(ClientError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Niri closed the connection",
            )));
        }
        if byte[0] == b'\n' {
            break;
        }
        if bytes.len() >= MAX_FRAME_BYTES {
            return Err(ClientError::Wire(WireError::TooLarge(MAX_FRAME_BYTES)));
        }
        bytes.push(byte[0]);
    }

    String::from_utf8(bytes).map_err(|_| ClientError::Wire(WireError::InvalidUtf8))
}

/// Discover the Niri socket from the current session environment.
///
/// Returns `None` when `NIRI_SOCKET` is unset, empty or not absolute.
pub fn socket_from_env() -> Option<PathBuf> {
    let value = std::env::var_os("NIRI_SOCKET")?;
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    if path.is_absolute() {
        Some(path)
    } else {
        None
    }
}

/// A client failure.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// The socket could not be opened.
    #[error("could not connect to the Niri socket: {0}")]
    Connect(#[source] std::io::Error),
    /// Communication failed after connecting.
    #[error("could not communicate with Niri: {0}")]
    Io(#[source] std::io::Error),
    /// The reply or event was invalid.
    #[error(transparent)]
    Wire(#[from] WireError),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::thread::JoinHandle;

    use nalios_core::niri::WindowId;
    use tempfile::TempDir;

    use super::*;

    fn fake_server(responses: Vec<String>) -> (TempDir, PathBuf, JoinHandle<()>) {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("niri.sock");
        let listener = UnixListener::bind(&path).expect("bind");

        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut writer = stream;

            for response in responses {
                let mut request = String::new();
                if reader.read_line(&mut request).expect("read request") == 0 {
                    return;
                }
                let _ = writer.write_all(response.as_bytes());
                let _ = writer.write_all(b"\n");
                let _ = writer.flush();
            }
        });

        (dir, path, handle)
    }

    /// The event stream sends one request and then receives many lines.
    fn fake_event_server(responses: Vec<String>) -> (TempDir, PathBuf, JoinHandle<()>) {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("niri.sock");
        let listener = UnixListener::bind(&path).expect("bind");

        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut writer = stream;

            let mut request = String::new();
            let _ = reader.read_line(&mut request);

            for response in responses {
                let _ = writer.write_all(response.as_bytes());
                let _ = writer.write_all(b"\n");
                let _ = writer.flush();
            }
        });

        (dir, path, handle)
    }

    #[test]
    fn snapshot_reads_all_supported_state() {
        let responses = vec![
            r#"{"Ok":{"Workspaces":[{"id":1,"idx":1,"name":null,"output":"eDP-1","is_urgent":false,"is_active":true,"is_focused":true,"active_window_id":null}]}}"#.to_owned(),
            r#"{"Ok":{"Windows":[{"id":5,"title":"Terminal","app_id":"alacritty","workspace_id":1,"is_focused":true,"is_floating":false,"is_urgent":false}]}}"#.to_owned(),
            r#"{"Ok":{"Outputs":{"eDP-1":{"name":"eDP-1","logical":{"x":0,"y":0,"width":1920,"height":1080,"scale":1.0,"transform":"normal"}}}}}"#.to_owned(),
            r#"{"Ok":{"FocusedWindow":null}}"#.to_owned(),
        ];
        let (_dir, path, handle) = fake_server(responses);

        let mut client = NiriClient::connect(&path).expect("connect");
        let snapshot = client.snapshot().expect("snapshot");

        handle.join().expect("server");
        assert_eq!(snapshot.workspaces.len(), 1);
        assert_eq!(snapshot.windows.len(), 1);
        assert_eq!(snapshot.outputs.len(), 1);
        assert!(snapshot.focused_window.is_none());
        assert_eq!(
            snapshot.outputs.values().next().expect("output").name.as_str(),
            "eDP-1"
        );
    }

    #[test]
    fn remote_errors_are_reported() {
        let (_dir, path, handle) = fake_server(vec![r#"{"Err":"no such workspace"}"#.to_owned()]);

        let mut client = NiriClient::connect(&path).expect("connect");
        let error = client.workspaces().expect_err("remote error");

        handle.join().expect("server");
        assert!(matches!(
            error,
            ClientError::Wire(WireError::Remote(message)) if message == "no such workspace"
        ));
    }

    #[test]
    fn oversized_replies_are_rejected() {
        let huge = "x".repeat(MAX_FRAME_BYTES + 1);
        let (_dir, path, handle) = fake_server(vec![huge]);

        let mut client = NiriClient::connect(&path).expect("connect");
        let error = client.workspaces().expect_err("too large");

        handle.join().expect("server");
        assert!(matches!(
            error,
            ClientError::Wire(WireError::TooLarge(MAX_FRAME_BYTES))
        ));
    }

    #[test]
    fn closed_connection_is_reported() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("niri.sock");
        let listener = UnixListener::bind(&path).expect("bind");

        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            drop(stream);
        });

        let mut client = NiriClient::connect(&path).expect("connect");
        let error = client.workspaces().expect_err("eof");

        handle.join().expect("server");
        assert!(matches!(error, ClientError::Io(_)));
    }

    #[test]
    fn missing_socket_is_a_connect_error() {
        let error = NiriClient::connect(Path::new("/definitely/missing/niri.sock"))
            .expect_err("connect error");

        assert!(matches!(error, ClientError::Connect(_)));
    }

    #[test]
    fn event_stream_skips_acknowledgements_and_unknown_events() {
        let responses = vec![
            r#"{"Ok":"Handled"}"#.to_owned(),
            r#"{"KeyboardLayoutsChanged":{"keyboard_layouts":{"names":["us"],"current_idx":0}}}"#
                .to_owned(),
            r#"{"WindowClosed":{"id":7}}"#.to_owned(),
        ];
        let (_dir, path, handle) = fake_event_server(responses);

        let mut stream = NiriEventStream::connect(&path).expect("connect");
        let event = stream.next_event().expect("event");

        handle.join().expect("server");
        assert_eq!(event, NiriEvent::WindowClosed { id: WindowId(7) });
    }
}
