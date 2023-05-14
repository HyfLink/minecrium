//! the fundamental crate for the minecrium.
//!
//! # Overview
//!
//! - [`coords`] defines the minecrium coordinate system.
//! - [`dynamic`] defines dynamic operations for trait objects.
//! - [`errors`] defines error types for the crate.
//! - [`resource`] resource identifaction and the registry.

// extern crates
pub extern crate cgmath;

// modules
pub mod coords;
pub mod dynamic;
pub mod errors;
pub mod resource;
