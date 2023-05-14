//! Defines error types for the [`minecrium_common`](crate) crate.

use std::error::Error as StdError;
use std::fmt;

/// An error that is [`<Axis as FromStr>::Err`](std::str::FromStr::Err).
#[derive(Clone, Copy, Debug, Default)]
pub struct ParseAxisError;

impl fmt::Display for ParseAxisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(r#"expects one of "z", "x", "y""#)
    }
}

impl StdError for ParseAxisError {}

/// An error that is [`<HAxis as FromStr>::Err`](std::str::FromStr::Err).
#[derive(Clone, Copy, Debug, Default)]
pub struct ParseHAxisError;

impl fmt::Display for ParseHAxisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(r#"expects one of "z", "x""#)
    }
}

impl StdError for ParseHAxisError {}

/// An error that is [`<Direction as FromStr>::Err`](std::str::FromStr::Err).
#[derive(Clone, Copy, Debug, Default)]
pub struct ParseDirectionError;

impl fmt::Display for ParseDirectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(r#"expects one of "south", "north", "east", "west", "up", "down"."#)
    }
}

impl StdError for ParseDirectionError {}

/// An error that is [`<HDirection as FromStr>::Err`](std::str::FromStr::Err).
#[derive(Clone, Copy, Debug, Default)]
pub struct ParseHDirectionError;

impl fmt::Display for ParseHDirectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(r#"expects one of "south", "north", "east", "west", "southeast", "southwest", "northeast", "northwest"."#)
    }
}

impl StdError for ParseHDirectionError {}

/// An error type for [`resource locations`](crate::resource::ResLocation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResLocationError {
    /// Resource location has an empty namespace.
    NamespaceEmpty,
    /// The namespace has non [a-z0-9_.-] character.
    NamespaceError,
    /// Resource location has an empty path.
    PathEmpty,
    /// The path has non [a-z0-9_.-/] character.
    PathError,
}

impl ResLocationError {
    /// Returns an error if the resource location namespace is invalid.
    ///
    /// A valid resource location namespace is:
    ///
    /// - non-empty, ascii-only, matches pattern `[a-z0-9_.-]+`
    pub fn check_namespace(namespace: &str) -> Result<(), Self> {
        #[inline(always)]
        fn is_valid_char(c: &u8) -> bool {
            matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.')
        }

        if namespace.is_empty() {
            Err(Self::NamespaceEmpty)
        } else if !namespace.as_bytes().iter().all(is_valid_char) {
            Err(Self::NamespaceError)
        } else {
            Ok(())
        }
    }

    /// Returns an error if the resource location path is invalid.
    ///
    /// A valid resource location path is:
    ///
    /// - non-empty, ascii-only, matches pattern `[a-z0-9_.-/]+`
    pub fn check_path(path: &str) -> Result<(), Self> {
        #[inline(always)]
        fn is_valid_char(c: &u8) -> bool {
            matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/')
        }

        if path.is_empty() {
            Err(Self::PathEmpty)
        } else if !path.as_bytes().iter().all(is_valid_char) {
            Err(Self::PathError)
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for ResLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NamespaceEmpty => "the resource location namespace is empty",
            Self::NamespaceError => "the resource location namespace has non [a-z0-9_.-] char",
            Self::PathEmpty => "the resource location path is empty",
            Self::PathError => "the resource location path has non [a-z0-9_.-/] char",
        })
    }
}

impl StdError for ResLocationError {}
