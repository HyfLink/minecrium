//! The minecrium uses the *right-handed coordinate system*, the relationship between direction
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

mod aabb;
mod coords;

// re-exports
pub use self::aabb::*;
pub use self::coords::*;
