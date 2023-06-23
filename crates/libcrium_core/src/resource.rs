use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// `ResLocation` (short for *resource location*) is a unique identifier to identifies
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
/// # use std::str::FromStr;
/// # use libcrium_core::resource::ResLocation;
/// #
/// // creates resource locations.
/// let loc0 = ResLocation::from_str("minecraft:dirt").unwrap();
/// let loc1 = ResLocation::new("minecraft", "dirt").unwrap();
/// assert_eq!(loc0, loc1);
///
/// assert_eq!(loc0.as_str(),    "minecraft:dirt");
/// assert_eq!(loc0.namespace(), "minecraft");
/// assert_eq!(loc0.path(),      "dirt");
/// ```
///
/// # Reference
///
/// - <https://docs.minecraftforge.net/en/latest/concepts/resources/>
#[derive(Clone)]
pub struct ResLocation {
    inner: Arc<ResLocationInner>,
}

struct ResLocationInner {
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
    location: Box<str>,
    delimiter: usize,
    /// the pre-computed hash of the resource location.
    hash: u64,
}

impl ResLocation {
    /// The default resource location namespace (`"minecrium"`).
    pub const DEFAULT_NAMESPACE: &str = "minecrium";

    /// Returns an resource location from the given namespace and path.
    ///
    /// Both the namespace and path are required to be **non-empty** and **ascii-only**.
    ///
    /// - the namespace should match the pattern `[a-z0-9_.-]+`.
    /// - the path should match the parrern `[a-z0-9_.-/]+`.
    ///
    /// # Errors
    ///
    /// Returns an error if the namespace or path is invalid.
    pub fn new(namespace: &str, path: &str) -> Result<Self, ResLocationError> {
        // checks the namespace and path.
        ResLocationError::check_namespace(namespace)?;
        ResLocationError::check_path(path)?;

        // SAFETY: the namespace and path are just checked.
        Ok(unsafe { Self::new_unchecked(namespace, path) })
    }

    /// Returns an resource location from the given path and the default namespace (`"minecrium"`).
    ///
    /// The path is required to be **non-empty**, **ascii-only** and should match the pattern
    /// `[a-z0-9_.-/]+`.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid.
    pub fn with_default_namespace(path: &str) -> Result<Self, ResLocationError> {
        // checks the path.
        ResLocationError::check_path(path)?;

        // SAFETY: the path is just checked, and the default namespace is always valid.
        Ok(unsafe { Self::new_unchecked(Self::DEFAULT_NAMESPACE, path) })
    }

    /// Returns an resource location from the given namespace and path without checking.
    ///
    /// # Safety
    ///
    /// Both `ResLocationError::check_namespace(namespace)` and `ResLocationError::check_path(path)`
    /// return `Ok`.
    unsafe fn new_unchecked(namespace: &str, path: &str) -> Self {
        // constructs the location mannually.
        let capacity = namespace.len() + path.len() + 1;
        let mut location = String::with_capacity(capacity);
        location.push_str(namespace);
        location.push(':');
        location.push_str(path);

        Self {
            inner: Arc::new(ResLocationInner {
                hash: hashes(&*location),
                location: location.into_boxed_str(),
                delimiter: namespace.len(),
            }),
        }
    }

    /// Returns the resource location as the string slice.
    ///
    /// Format: `"{namespace}:{path}"`.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.inner.location
    }

    /// Returns the namespace (`.0`) and path (`.1`) of the resource location.
    #[inline]
    pub fn as_parts(&self) -> (&str, &str) {
        let inner = self.inner.as_ref();
        (
            &inner.location[..inner.delimiter],
            &inner.location[inner.delimiter + 1..],
        )
    }

    /// Returns the namespace of the resource location.
    #[inline]
    pub fn namespace(&self) -> &str {
        let inner = self.inner.as_ref();
        &inner.location[..inner.delimiter]
    }

    /// Returns the path of the resource location.
    #[inline]
    pub fn path(&self) -> &str {
        let inner = self.inner.as_ref();
        &inner.location[inner.delimiter + 1..]
    }
}

impl PartialEq for ResLocation {
    fn eq(&self, other: &Self) -> bool {
        let this = self.inner.as_ref();
        let other = other.inner.as_ref();
        this.hash == other.hash && this.location == other.location
    }
}

impl Eq for ResLocation {}

impl Hash for ResLocation {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash.hash(state);
    }
}

impl fmt::Debug for ResLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (namespace, path) = self.as_parts();
        f.debug_struct("ResLocation")
            .field("namespace", &namespace)
            .field("path", &path)
            .finish()
    }
}

impl fmt::Display for ResLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.inner.location.as_ref())
    }
}

impl AsRef<str> for ResLocation {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<&str> for ResLocation {
    type Error = ResLocationError;
    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for ResLocation {
    type Error = ResLocationError;
    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.into_boxed_str())
    }
}

impl TryFrom<Box<str>> for ResLocation {
    type Error = ResLocationError;

    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        if let Some((namespace, path)) = value.split_once(':') {
            // checks the namespace and path.
            ResLocationError::check_namespace(namespace)?;
            ResLocationError::check_path(path)?;

            Ok(Self {
                inner: Arc::new(ResLocationInner {
                    hash: hashes(&*value),
                    delimiter: namespace.len(),
                    location: value,
                }),
            })
        } else {
            // there is no delimiter ':'.
            // constructs the resource location with the default namespace.
            Self::with_default_namespace(&value)
        }
    }
}

impl TryFrom<Cow<'_, str>> for ResLocation {
    type Error = ResLocationError;
    #[inline]
    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        match value {
            Cow::Borrowed(s) => Self::try_from(s),
            Cow::Owned(s) => Self::try_from(s),
        }
    }
}

impl FromStr for ResLocation {
    type Err = ResLocationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((namespace, path)) = s.split_once(':') {
            Self::new(namespace, path)
        } else {
            // there is no delimiter ':'.
            // constructs the resource location with the default namespace.
            Self::with_default_namespace(s)
        }
    }
}

impl Serialize for ResLocation {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        <str as Serialize>::serialize(self.as_ref(), serializer)
    }
}

impl<'de> Deserialize<'de> for ResLocation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let location = <Box<str> as Deserialize<'de>>::deserialize(deserializer)?;
        match Self::try_from(location) {
            Ok(location) => Ok(location),
            Err(err) => Err(<D::Error as serde::de::Error>::custom(err)),
        }
    }
}

/// An error when creating [`resource locations`](ResLocation).
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

/// Returns the hash of the value. Used to compute the `ResLocationInner.hash`.
fn hashes<T: ?Sized + Hash>(value: &T) -> u64 {
    let builder = bevy::utils::FixedState;
    let mut hasher = builder.build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use crate::resource::*;

    #[test]
    fn test_resource_location() {
        assert!(ResLocationError::check_namespace(ResLocation::DEFAULT_NAMESPACE).is_ok());

        assert!(ResLocation::from_str("minecraft:dirt").is_ok());
        assert!(ResLocation::from_str("mine_craft0:soul_sand_0").is_ok());
        assert!(ResLocation::from_str("mine-craft1:ore/coal-ore-1").is_ok());
        assert!(ResLocation::from_str("mine.craft2:ore/coal.ore.2").is_ok());

        assert_eq!(
            ResLocation::from_str("mine_craft0:soul_sand_0").unwrap(),
            ResLocation::new("mine_craft0", "soul_sand_0").unwrap(),
        );

        assert_eq!(
            ResLocation::from_str("minecraft:"),
            Err(ResLocationError::PathEmpty)
        );

        assert_eq!(
            ResLocation::from_str(":dirt"),
            Err(ResLocationError::NamespaceEmpty)
        );

        assert_eq!(
            ResLocation::from_str(":"),
            Err(ResLocationError::NamespaceEmpty)
        );

        assert_eq!(
            ResLocation::from_str("Minecraft:dirt"),
            Err(ResLocationError::NamespaceError)
        );

        assert_eq!(
            ResLocation::from_str("minecraft:Dirt"),
            Err(ResLocationError::PathError)
        );
    }
}
