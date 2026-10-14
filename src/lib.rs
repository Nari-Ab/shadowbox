//! ShadowBox: Zero-trust, copy-on-write runtime and interactive time-machine for AI coding agents.

pub mod diff;
pub mod runner;
pub mod session;
pub mod shield;
pub mod tui;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
