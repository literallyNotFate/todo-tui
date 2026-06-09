///
/// Toodles - TUI task manager written in Rust
///
pub mod app;
pub mod application;
pub mod common;
pub mod config;
pub mod core;
pub mod events;
pub mod models;
pub mod state;
pub mod theme;
pub mod ui;

pub use application::Application;

/// Pretty assertions for tests
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
