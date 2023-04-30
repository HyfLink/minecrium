use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

use bevy::utils::Hashed;
use serde::{Deserialize, Serialize};

/// `ResLoc` (short for *resource location*) is a unique identifier to identifies
/// resources.
///
/// The resource location consists of two parts, `namespace` and `path`. It
/// points to the resource at `assets/<namespace>/<ctx>/<path>`, where `<ctx>`
/// is a context-specified path fragment.
///
/// - both of the `nampespace` and `path` are required to be *non-empty* and
/// *ascii-only*
///
/// - the `namespace` is required to match the pattern `[a-z0-9_.-]+`.
///
/// - the `path` is required to match the pattern `[a-z0-9_.-/]+`.
///
/// # Examples
///
/// ```
/// # use minecrium_core::resource::ResLoc;
/// #
/// // creates resource locations.
/// let loc = ResLoc::new("minecraft:dirt").unwrap();
/// let loc1 = ResLoc::from_parts("minecraft", "dirt").unwrap();
/// assert_eq!(loc, loc1);
///
/// assert_eq!(loc.as_str(),    "minecraft:dirt");
/// assert_eq!(loc.namespace(), "minecraft");
/// assert_eq!(loc.path(),      "dirt");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ResLoc(Arc<ResLocInner>);

/// Implementation of `ResLoc`.
///
/// the following graph shows the relation between `self.location` and
/// `self.delimiter`.
///
/// ```text
///     "minecraft:redstone_torch"
///     ^^        ^^
///     ||        ||
///     ||        |path="redstone_torch"
///     ||        |
///     ||       delimiter=9 (= namespace.len())
///     ||
///     |namespace="minecraft"
///     |
///    location="minecraft:redstone_torch"
/// ```
#[derive(PartialEq, Eq, Hash)]
struct ResLocInner {
    delimiter: usize,
    location: Hashed<String>,
}

impl ResLoc {
    /// Creates a new resource location from the given string.
    ///
    /// The string should be of format `{namespace}:{path}`.
    ///
    /// # Errors
    ///
    /// Returns [`ResLocError`] if the `loc` does not contain a colon (`':'`),
    /// or if the `namespace` or `path` is invalid (see [`ResLoc`]).
    pub fn new<S: Into<String>>(loc: S) -> Result<Self, ResLocError> {
        let loc = loc.into();

        if let Some(delimiter) = loc.find(':') {
            let (namespace, colon_path) = loc.split_at(delimiter);
            let path = &colon_path[1..];
            ResLocError::check_namespace(namespace)?;
            ResLocError::check_path(path)?;

            Ok(Self(Arc::new(ResLocInner {
                delimiter,
                location: Hashed::new(loc),
            })))
        } else {
            Err(ResLocError::NamespaceEmpty)
        }
    }

    /// Creates a new resource location from the given namespace and path.
    ///
    /// # Errors
    ///
    /// Returns [`ResLocError`] if the `namespace` or `path` is invalid (see
    /// [`ResLoc`]).
    pub fn from_parts(namespace: &str, path: &str) -> Result<Self, ResLocError> {
        ResLocError::check_namespace(namespace)?;
        ResLocError::check_path(path)?;
        let loc = format!("{namespace}:{path}");

        Ok(Self(Arc::new(ResLocInner {
            delimiter: namespace.len(),
            location: Hashed::new(loc),
        })))
    }

    /// Returns the formatted resource location (`{namespace}:{path}`).
    pub fn as_str(&self) -> &str {
        &self.0.location
    }

    /// Returns the namespace of this resource location.
    pub fn namespace(&self) -> &str {
        &self.0.location[..self.0.delimiter]
    }

    /// Returns the path of this resource location.
    pub fn path(&self) -> &str {
        &self.0.location[self.0.delimiter + 1..]
    }
}

impl Debug for ResLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as Debug>::fmt(self.as_str(), f)
    }
}

impl Display for ResLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as Display>::fmt(self.as_str(), f)
    }
}

impl Serialize for ResLoc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <String as Serialize>::serialize(&*self.0.location, serializer)
    }
}

impl<'de> Deserialize<'de> for ResLoc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match ResLoc::new(<String as Deserialize>::deserialize(deserializer)?) {
            Ok(loc) => Ok(loc),
            Err(err) => Err(serde::de::Error::custom(err)),
        }
    }
}

/// Errors when creating resource location.
#[derive(Clone)]
pub enum ResLocError {
    /// Resource location has an empty namespace.
    NamespaceEmpty,
    /// The namespace has non [a-z0-9_.-] character.
    NamespaceError,
    /// Resource location has an empty path.
    PathEmpty,
    /// The path has non [a-z0-9_.-/] character.
    PathError,
}

impl ResLocError {
    /// A valid resource location namespace is:
    ///
    /// - non-empty
    /// - ascii-only
    /// - matches pattern `[a-z0-9_.-]+`
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

    /// A valid resource location path is:
    ///
    /// - non-empty
    /// - ascii-only
    /// - matches pattern `[a-z0-9_.-/]+`
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

impl Debug for ResLocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NamespaceEmpty => "the namespace is empty",
            Self::NamespaceError => "the resource location namespace has non [a-z0-9_.-] char",
            Self::PathEmpty => "the path is empty",
            Self::PathError => "the resource location path has non [a-z0-9_.-/] char",
        })
    }
}

impl Display for ResLocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl StdError for ResLocError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_location_new() {
        #[rustfmt::skip]
        assert!(ResLoc::new("minecraft:dirt").is_ok());

        #[rustfmt::skip]
        assert!(ResLoc::new("mine_craft0:soul_sand_0").is_ok());

        assert_eq!(
            ResLoc::new("mine_craft0:soul_sand_0").unwrap(),
            ResLoc::from_parts("mine_craft0", "soul_sand_0").unwrap(),
        );

        #[rustfmt::skip]
        assert!(ResLoc::new("mine-craft1:ore/coal-ore-1").is_ok());

        #[rustfmt::skip]
        assert!(ResLoc::new("mine.craft2:ore/coal.ore.2").is_ok());

        #[rustfmt::skip]
        assert!(matches!(ResLoc::new("minecraft:"), Err(ResLocError::PathEmpty)));

        #[rustfmt::skip]
        assert!(matches!(ResLoc::new(":dirt"), Err(ResLocError::NamespaceEmpty)));

        #[rustfmt::skip]
        assert!(matches!(ResLoc::new(":"), Err(ResLocError::NamespaceEmpty)));

        #[rustfmt::skip]
        assert!(matches!(ResLoc::new("Minecraft:dirt"), Err(ResLocError::NamespaceError)));

        #[rustfmt::skip]
        assert!(matches!(ResLoc::new("minecraft:Dirt"), Err(ResLocError::PathError)));
    }
}
