//! Provides utilities for minecrium proc-macro crates.

mod manifest;
mod rename;

// re-exports
pub use manifest::get_crate_path;
pub use rename::RenameStyle;
