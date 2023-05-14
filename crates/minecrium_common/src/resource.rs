//! Defines the resource registry and resource identification.
//!
//! # Overview
//!
//! | items                    | description                                                       |
//! | ------------------------ | ----------------------------------------------------------------- |
//! | [`Registry`]             | A collection to manage resources.                                 |
//! | [`ResKey`]               | An index to the registry.                                         |
//! | [`ResLocation`]          | A unique identifier for resources.                                |
//!
//! # Reference
//!
//! - <https://docs.minecraftforge.net/en/latest/concepts/resources/>

use std::borrow::Cow;
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use std::sync::Arc;

use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

use crate::errors::ResLocationError;

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
/// # use minecrium_common::resource::ResLocation;
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
pub struct ResLocation {
    inner: Arc<ResLocationInner>,
}

struct ResLocationInner {
    /// the pre-computed hash of the resource location.
    hash: u64,
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
}

impl Clone for ResLocation {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
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

/// A specialized index to the [`Registry<T>`].
///
/// [`Registry<T>`] is randomly accessile by [`ResKey<T>`].
pub struct ResKey<T> {
    index: u32,
    marker: PhantomData<T>,
}

impl<T> Clone for ResKey<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            marker: PhantomData,
        }
    }
}

impl<T> Copy for ResKey<T> {}

impl<T> PartialEq for ResKey<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for ResKey<T> {}

impl<T> PartialOrd for ResKey<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for ResKey<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Hash for ResKey<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> fmt::Debug for ResKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <u32 as fmt::Debug>::fmt(&self.index, f)
    }
}

impl<T> fmt::Display for ResKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <u32 as fmt::Display>::fmt(&self.index, f)
    }
}

impl<T> From<ResKey<T>> for u32 {
    #[inline]
    fn from(value: ResKey<T>) -> Self {
        value.index
    }
}

impl<T> From<ResKey<T>> for usize {
    #[inline]
    fn from(value: ResKey<T>) -> Self {
        value.index as usize
    }
}

impl<T> From<u32> for ResKey<T> {
    #[inline]
    fn from(value: u32) -> Self {
        Self {
            index: value,
            marker: PhantomData,
        }
    }
}

/// A specialized hash map with the keys of [`ResLocation`] and the values of `T`.
///
/// The container also provides random access with the index of [`ResKey<T>`].
pub struct Registry<T> {
    /// Maps resource key to resource value.
    store: Vec<T>,
    /// Maps resource location to resource key.
    index: HashMap<ResLocation, u32>,
}

impl<T> Registry<T> {
    /// Returns an empty registry.
    #[inline]
    pub fn new() -> Self {
        Self {
            store: Vec::new(),
            index: HashMap::default(),
        }
    }

    /// Returns an empty registry with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: Vec::with_capacity(capacity),
            index: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the number of elements in the registry.
    #[inline]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns `true` if the registry contains no element.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Returns a slice containing all the elements in the registry.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.store
    }

    /// Returns an unordered iterator over resource locations, resource keys and values of the
    /// elements.
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            iter: self.index.iter(),
            slice: &self.store,
        }
    }

    /// Returns an unordered iterator over resource locations of the elements.
    #[inline]
    pub fn keys(&self) -> Keys<'_, T> {
        Keys {
            iter: self.index.keys(),
            marker: PhantomData,
        }
    }

    /// Returns an iterator over resource locations of the elements.
    #[inline]
    pub fn values(&self) -> Values<'_, T> {
        Values {
            iter: self.store.iter(),
        }
    }

    /// Returns `true` if the registry contains an element corresponding to the resource key.
    #[inline]
    pub fn contains_key(&self, key: ResKey<T>) -> bool {
        (key.index as usize) < self.store.len()
    }

    /// Returns `true` if the registry contains an element corresponding to the resource location.
    #[inline]
    pub fn contains_loc(&self, loc: &ResLocation) -> bool {
        self.index.contains_key(loc)
    }

    /// Returns the reference to the element corresponding to the given resource key.
    #[inline]
    pub fn get(&self, key: ResKey<T>) -> Option<&T> {
        self.store.get(usize::from(key))
    }

    /// Returns the mutable reference to the element corresponding to the given resource key.
    #[inline]
    pub fn get_mut(&mut self, key: ResKey<T>) -> Option<&mut T> {
        self.store.get_mut(usize::from(key))
    }

    /// Returns the reference to the element corresponding to the given resource location.
    #[inline]
    pub fn get_by_loc(&self, loc: &ResLocation) -> Option<&T> {
        let key = self.index.get(loc)?;
        self.store.get(*key as usize)
    }

    /// Returns the mutable reference to the element corresponding to the given resource location.
    #[inline]
    pub fn get_mut_by_loc(&mut self, loc: &ResLocation) -> Option<&mut T> {
        let key = self.index.get(loc)?;
        self.store.get_mut(*key as usize)
    }

    /// Returns the resource key corresponding to the resource location.
    #[inline]
    pub fn get_key(&self, loc: &ResLocation) -> Option<ResKey<T>> {
        let key = self.index.get(loc)?;
        Some(ResKey::from(*key))
    }

    /// Insert the resource location and the value into the registry. Returns a resource key
    /// corresponding to the newly inserted value.
    ///
    /// # Errors
    ///
    /// Returns the `value` as an error if `self.contains_loc(&loc)`.
    ///
    /// # Panics
    ///
    /// Panics if the registy contains too many elements (`self.len() > u32::MAX`).
    pub fn insert(&mut self, loc: ResLocation, value: T) -> Result<ResKey<T>, T> {
        if !self.contains_loc(&loc) {
            // SAFETY: the `loc` is just checked.
            Ok(unsafe { self.insert_unique_unchecked(loc, value) })
        } else {
            Err(value)
        }
    }

    /// Insert the resource location and the value into the registry without checking if the key
    /// already exists in the map. Returns a resource key corresponding to the newly inserted value.
    ///
    /// # Safety
    ///
    /// This method is safe if `!self.contains_loc(&loc)`.
    ///
    /// # Panics
    ///
    /// Panics if the registy contains too many elements (`self.len() > u32::MAX`).
    pub unsafe fn insert_unique_unchecked(&mut self, loc: ResLocation, value: T) -> ResKey<T> {
        let key = ResKey::from(u32::try_from(self.store.len()).unwrap());
        self.store.push(value);
        self.index.insert_unique_unchecked(loc, u32::from(key));
        key
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the registry.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.store.reserve(additional);
        self.index.reserve(additional);
    }
}

impl<T> Default for Registry<T> {
    #[inline]
    fn default() -> Self {
        Self {
            store: Vec::new(),
            index: HashMap::default(),
        }
    }
}

impl<T: Clone> Clone for Registry<T> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            index: self.index.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Registry<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = f.debug_map();

        for (loc, key) in self.index.iter() {
            state.entry(loc, self.store.index(*key as usize));
        }

        state.finish()
    }
}

impl<T> AsRef<[T]> for Registry<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.store
    }
}

impl<T> AsMut<[T]> for Registry<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.store
    }
}

impl<T> Index<ResKey<T>> for Registry<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: ResKey<T>) -> &Self::Output {
        self.store.index(usize::from(index))
    }
}

impl<T> IndexMut<ResKey<T>> for Registry<T> {
    #[inline]
    fn index_mut(&mut self, index: ResKey<T>) -> &mut Self::Output {
        self.store.index_mut(usize::from(index))
    }
}

impl<T> Index<&ResLocation> for Registry<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: &ResLocation) -> &Self::Output {
        let index = self.index.index(index);
        self.store.index(*index as usize)
    }
}

impl<T> IndexMut<&ResLocation> for Registry<T> {
    #[inline]
    fn index_mut(&mut self, index: &ResLocation) -> &mut Self::Output {
        let index = self.index.index(index);
        self.store.index_mut(*index as usize)
    }
}

/// Returns the hash of the value. Used to compute the `ResLocationInner.hash`.
fn hashes<T: ?Sized + Hash>(value: &T) -> u64 {
    let builder = bevy::utils::FixedState;
    let mut hasher = builder.build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

/// An iterator that is returned by `Registry::iter`.
pub struct Iter<'a, T> {
    iter: bevy::utils::hashbrown::hash_map::Iter<'a, ResLocation, u32>,
    slice: &'a [T],
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            slice: self.slice,
        }
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Iter<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = f.debug_map();

        for (key, _, value) in self.clone() {
            state.entry(key, value);
        }

        state.finish()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a ResLocation, ResKey<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (loc, &key) = self.iter.next()?;
        let value = self.slice.index(key as usize);
        let key = ResKey::from(key);
        Some((loc, key, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T> FusedIterator for Iter<'a, T> {}

/// An iterator that is returned by `Registry::keys`.
pub struct Keys<'a, T> {
    iter: bevy::utils::hashbrown::hash_map::Keys<'a, ResLocation, u32>,
    marker: PhantomData<T>,
}

impl<'a, T> Clone for Keys<'a, T> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            marker: PhantomData,
        }
    }
}

impl<'a, T> fmt::Debug for Keys<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.clone()).finish()
    }
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = &'a ResLocation;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Keys<'a, T> {}

impl<'a, T> FusedIterator for Keys<'a, T> {}

/// An iterator that is returned by `Registry::values`.
pub struct Values<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

impl<'a, T> Clone for Values<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Values<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.clone()).finish()
    }
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Values<'a, T> {}

impl<'a, T> FusedIterator for Values<'a, T> {}

#[cfg(test)]
mod tests {
    use crate::resource::*;

    #[test]
    fn test_resource_location() {
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
