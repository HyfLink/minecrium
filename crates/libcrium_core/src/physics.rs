//! The minecrium uses the **right-handed coordinate system**, the relationship between direction
//! and axis is demonstrated in the following graph.
//!
//! ```text
//!               up(Y)
//!                 |   north
//!                 |  /
//!                 | /
//!                 |/
//! west <----------+----------> east(X)
//!                /|
//!               / |
//!              /  |
//!        south(Z) |
//!                 down
//! ```
//!
//! A minecrium chunk is a regualr quadriprism, with an bottom edge length of `16` (in blocks) and
//! a customized height.

mod aabb;
mod direction;

// re-exports
pub use self::aabb::Aabb;
pub use self::direction::*;
