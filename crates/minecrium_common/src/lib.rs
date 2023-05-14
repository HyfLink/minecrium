//! the fundamental crate for the minecrium.
//!
//! # Overview
//!
//! - [`coords`] defines the minecrium coordinate system.
//! - [`errors`] defines error types for the crate.
//! - [`resource`] resource identifaction and the registry.

// extern crates
pub extern crate cgmath;

// modules
pub mod coords;
pub mod errors;
pub mod resource;
