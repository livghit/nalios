//! NaliOS platform adapters.
//!
//! This crate implements application ports with real Linux, Niri and
//! filesystem effects. It owns no product workflow and exposes no widgets.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod apps;
pub mod control;
pub mod diagnostics;
pub mod files;
pub mod niri;
