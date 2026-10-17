//! NaliOS pure domain boundary.
//!
//! This crate owns validated values and rules with no I/O, runtime or toolkit
//! dependencies. Modules are added as their implementation tickets begin.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod apps;
pub mod config;
pub mod error;
pub mod niri;
pub mod paths;

pub use error::Error;
