//! Control socket worker for the shell.
//!
//! The socket accept loop is blocking, so it runs on a dedicated thread and
//! forwards allowlisted actions to the UI through a bounded channel.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use nalios_app::control::{ControlAction, ControlResponse};
use nalios_platform::control::ControlServer;

/// Owns the control socket thread and its action channel.
pub struct ControlWorker {
    stop: Arc<AtomicBool>,
    actions: Receiver<ControlAction>,
}

impl ControlWorker {
    /// Bind the control socket and start accepting requests.
    ///
    /// If binding fails (for example another shell is already running), the
    /// error is reported on stderr and the shell continues without control.
    pub fn spawn() -> Self {
        let (sender, actions) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();

        thread::spawn(move || run(sender, thread_stop));

        Self { stop, actions }
    }

    /// Receive side for the UI poll loop.
    pub fn actions(&self) -> &Receiver<ControlAction> {
        &self.actions
    }

    /// Ask the accept loop to stop.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for ControlWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(sender: Sender<ControlAction>, stop: Arc<AtomicBool>) {
    let server = match ControlServer::bind() {
        Ok(server) => server,
        Err(error) => {
            eprintln!("nalios-shell: control socket unavailable: {error}");
            return;
        }
    };

    while !stop.load(Ordering::Relaxed) {
        let mut connection = match server.accept() {
            Ok(connection) => connection,
            Err(_) => break,
        };

        let action = match connection.receive() {
            Ok(request) => request.action,
            Err(error) => {
                let _ = connection.respond(&ControlResponse::Error {
                    message: error.to_string(),
                });
                continue;
            }
        };

        // Accept the request first; the UI performs it asynchronously.
        let _ = connection.respond(&ControlResponse::Ok);
        if sender.send(action).is_err() {
            return;
        }
    }
}
