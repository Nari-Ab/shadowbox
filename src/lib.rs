//! ShadowBox: Zero-trust, copy-on-write runtime and interactive time-machine for AI coding agents.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
