//! Independent client for the documented Niri IPC protocol.
//!
//! The client is authored from public protocol documentation and recorded
//! observations. It does not link the GPL `niri-ipc` crate and tolerates
//! unknown fields and events for forward compatibility.
//!
//! This module is synchronous. The headless CLI uses it directly; the shell
//! will integrate it through an explicitly owned worker or a later async
//! adapter rather than blocking its UI loop.

mod client;
mod wire;

pub use client::{socket_from_env, ClientError, NiriClient, NiriEventStream, MAX_FRAME_BYTES};
pub use wire::WireError;
