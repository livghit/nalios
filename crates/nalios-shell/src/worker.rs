//! Niri reader worker for the shell.
//!
//! The Niri event stream is blocking, so it runs on a dedicated thread and
//! forwards bounded messages to the UI. The UI polls the channel on a timer;
//! it never blocks on the socket.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

use nalios_app::niri::NiriEvent;
use nalios_platform::niri::{NiriEventStream, socket_from_env};

/// A message from the Niri reader thread to the UI.
pub enum NiriMessage {
    /// The stream connected; state is live.
    Connected,
    /// The stream disconnected; state is stale until reconnect.
    Disconnected,
    /// One modeled Niri event.
    Event(NiriEvent),
}

/// Owns the Niri reader thread and its message channel.
pub struct NiriWorker {
    stop: Arc<AtomicBool>,
    messages: Receiver<NiriMessage>,
}

impl NiriWorker {
    /// Spawn the reader thread.
    pub fn spawn() -> Self {
        let (sender, messages) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();

        thread::spawn(move || run(sender, thread_stop));

        Self { stop, messages }
    }

    /// Receive side for the UI poll loop.
    pub fn messages(&self) -> &Receiver<NiriMessage> {
        &self.messages
    }

    /// Ask the reader thread to stop.
    ///
    /// The thread may be blocked in a socket read; it exits after the current
    /// read returns. The process owns the thread until then.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for NiriWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(sender: Sender<NiriMessage>, stop: Arc<AtomicBool>) {
    let Some(path) = socket_from_env() else {
        let _ = sender.send(NiriMessage::Disconnected);
        return;
    };

    while !stop.load(Ordering::Relaxed) {
        match NiriEventStream::connect(&path) {
            Ok(mut stream) => {
                if sender.send(NiriMessage::Connected).is_err() {
                    return;
                }

                loop {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    match stream.next_event() {
                        Ok(event) => {
                            if sender.send(NiriMessage::Event(event)).is_err() {
                                return;
                            }
                        }
                        Err(_) => {
                            let _ = sender.send(NiriMessage::Disconnected);
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                let _ = sender.send(NiriMessage::Disconnected);
            }
        }

        // Bounded reconnect backoff that stays responsive to `stop`.
        for _ in 0..10 {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
