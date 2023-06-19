use std::any::{self, TypeId};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::{NonZeroU16, NonZeroUsize};
use std::ops::Range;

use bevy::utils::HashMap;
use libcrium_core::dynamic::{downcast_sync, AsAnySync, CastError};
use libcrium_core::primitive;
use libcrium_core::strenum::StrEnum;

// re-exports
pub use libcrium_block_macros::property;
pub use libcrium_core::strenum::{ReflectValue, Value, ValueUntyped};

/// Definition of the block state property, consists of the property key and value range.
///
/// # Generic Params
///
/// - `T` is the value type of the block property.
///
///   - [`bool`], for boolean block property.
///     the value range is always `false` and `true`.
///
///   - [`u8`], for integer block property.
///     the value range is continuous range of `u8` integers.
///
///   - enum implements [`StrEnum`], for enum block property.
///     the value range is a set of variants of the enum type.
///
/// # Examples
///
/// ```
/// # use libcrium_core::physics::Direction;
/// # use libcrium_block::property::Property;
/// // boolean property.
/// static WATERLOGGED: Property<bool> = Property::boolean("waterlogged");
/// // integer property.
/// static POWEREDNESS: Property<u8> = Property::integer("poweredness", 0..16);
/// // enum property.
/// static DIRECTION: Property<Direction> = Property::enums("direction");
/// // enum property with custom values.
/// static HORIZONTAL_DIRECTION: Property<Direction> = Property::enums_with("horizontal_direction", &[
///     Direction::South, Direction::North, Direction::East, Direction::West,
/// ]);
/// ```
#[derive(Clone, Copy)]
pub struct Property<T: Value> {
    /// The unique key of the block property.
    key: &'static str,
    /// A slice containing all the elements of the block property.
    range: &'static [T],
}

impl Property<bool> {
    /// Returns the boolean block property with the specified key.
    #[must_use]
    pub const fn boolean(key: &'static str) -> Self {
        Self {
            key,
            range: primitive::bool::sequence(),
        }
    }
}

impl Property<u8> {
    /// Returns the integer block property with the specified key.
    ///
    /// Panics if the `range` contains less than `2` elements.
    #[must_use]
    pub const fn integer(key: &'static str, range: Range<u8>) -> Self {
        if range.end - range.start < 2 {
            panic!("the integer property expects at least 2 values");
        }

        Self {
            key,
            range: primitive::u8::sequence(range),
        }
    }
}

impl<T: StrEnum> Property<T> {
    /// Returns the enum block property with the specified key.
    ///
    /// Panics if the enum `T` contains less than `2` variants.
    #[must_use]
    pub const fn enums(key: &'static str) -> Self {
        Self::enums_with(key, <T as StrEnum>::VALUES)
    }

    /// Returns the enum block property with the specified key.
    ///
    /// Panics if the `range` contains less than `2` elements.
    #[must_use]
    pub const fn enums_with(key: &'static str, range: &'static [T]) -> Self {
        if range.len() < 2 {
            panic!("the enum property expects at least 2 values");
        }

        Self { key, range }
    }
}

impl<T: Value> Property<T> {
    /// Returns a slice containing all the elements of the property.
    #[must_use]
    pub fn range(&self) -> &'static [T] {
        self.range
    }

    /// Returns an iterator over all the elements of the property.
    pub fn iter(&self) -> impl Iterator<Item = &'static T> {
        self.range.iter()
    }

    /// Returns `true` if the block property contains the `value`.
    #[must_use]
    pub fn contains(&self, value: &T) -> bool {
        trait Specialization<T> {
            fn call(&self, value: &T) -> bool;
        }

        impl<T: Value> Specialization<T> for Property<T> {
            default fn call(&self, value: &T) -> bool {
                self.range.contains(value)
            }
        }

        impl Specialization<bool> for Property<bool> {
            fn call(&self, _: &bool) -> bool {
                true
            }
        }

        impl Specialization<u8> for Property<u8> {
            fn call(&self, value: &u8) -> bool {
                let &[min, .., max] = self.range else { unreachable!() };
                min <= *value && *value <= max
            }
        }

        <Self as Specialization<T>>::call(self, value)
    }

    /// Returns the value that is converted from the non-generic value.
    ///
    /// Returns [`None`] if the conversion failds or if the block property does not contain the
    /// value.
    #[must_use]
    pub fn cast(&self, value: ValueUntyped<'_>) -> Option<&'static T> {
        trait Specialization<T> {
            fn call(&self, value: ValueUntyped<'_>) -> Option<&'static T>;
        }

        impl<T: Value> Specialization<T> for Property<T> {
            default fn call(&self, value: ValueUntyped<'_>) -> Option<&'static T> {
                if let Ok(value) = <T as Value>::from_value(value) {
                    self.range.iter().find(|elem| value.eq(elem))
                } else {
                    None
                }
            }
        }

        impl Specialization<bool> for Property<bool> {
            fn call(&self, value: ValueUntyped<'_>) -> Option<&'static bool> {
                if let ValueUntyped::Boolean(boolean) = value {
                    Some(if boolean { &true } else { &false })
                } else {
                    None
                }
            }
        }

        impl Specialization<u8> for Property<u8> {
            fn call(&self, value: ValueUntyped<'_>) -> Option<&'static u8> {
                if let ValueUntyped::Integer(integer) = value {
                    Some(&primitive::u8::SEQUENCE[integer as usize])
                } else {
                    None
                }
            }
        }

        <Self as Specialization<T>>::call(self, value)
    }
}

impl<T: Value> fmt::Debug for Property<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key)?;
        f.write_str("[")?;

        let mut iter = self.range.iter();

        if let Some(first) = iter.next() {
            f.write_str(first.as_str())?;
        }

        for next in iter {
            f.write_str(", ")?;
            f.write_str(next.as_str())?;
        }

        f.write_str("[")
    }
}

impl<T: Value> Hash for Property<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<T: Value> Eq for Property<T> {}

impl<T: Value> PartialEq for Property<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

/// The non-generic version of the [`Property<T>`], can be made into trait object.
///
/// # Implementors
///
/// The only implementor is [`Property<T>`].
// `self.len()` returns `NonZeroUsize` and is never empty.
#[allow(clippy::len_without_is_empty)]
pub trait ReflectProperty: AsAnySync + fmt::Debug {
    /// Returns the unique key of the block property.
    fn key(&self) -> &'static str;

    /// Returns the number of elements in the property.
    fn len(&self) -> NonZeroUsize;

    /// Tests for `self` and `other` values and types to be equal.
    fn dyn_eq(&self, other: &dyn ReflectProperty) -> bool;

    /// Returns an iterator over all the elements of the property.
    fn dyn_iter(&self) -> Box<dyn Iterator<Item = &'static dyn ReflectValue>>;

    /// Returns `true` if the block property contains the `value`.
    fn dyn_contains(&self, value: &dyn ReflectValue) -> bool;

    /// Returns the value that is converted from the non-generic value.
    ///
    /// Returns [`None`] if the conversion failds or if the block property does not contain the
    /// value.
    fn dyn_cast(&self, value: ValueUntyped<'_>) -> Option<&'static dyn ReflectValue>;
}

impl dyn ReflectProperty {
    /// Returns the downcast value as [`Property<T>`].
    ///
    /// Returns [`None`] if the inner type of `self` is not [`Property<T>`].
    #[must_use]
    pub fn downcast<T: Value>(&self) -> Option<Property<T>> {
        if self.type_id() == TypeId::of::<Property<T>>() {
            // SAFETY: the inner type of `other` is checked to be `Self`.
            Some(unsafe { *(self as *const _ as *const Property<T>) })
        } else {
            None
        }
    }
}

impl Hash for dyn ReflectProperty {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
        self.type_id().hash(state);
    }
}

impl Eq for dyn ReflectProperty {}

impl PartialEq for dyn ReflectProperty {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl<T: Value> PartialEq<Property<T>> for dyn ReflectProperty {
    fn eq(&self, other: &Property<T>) -> bool {
        matches!(self.downcast(), Some(value) if value.eq(other))
    }
}

impl<T: Value> PartialEq<dyn ReflectProperty> for Property<T> {
    fn eq(&self, other: &dyn ReflectProperty) -> bool {
        matches!(other.downcast(), Some(other) if other.eq(self))
    }
}

/// A group of [`Property`]s, represents a kind of block state.
///
/// # Implementors
///
/// - `()` represnets the empty block state.
///
/// - struct with attrubute [`macro@property`].
///
/// # Examples
///
/// Implements the trait through attrubute [`macro@property`].
///
/// ```
/// # use libcrium_block::property::{property, Property};
/// #
/// static FOO: Property<bool> = Property::boolean("foo");
/// static BAR: Property<u8> = Property::integer("bar", 0..5);
///
/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// #[property(crate = libcrium_block)]
/// pub struct FooBar {
///     #[property = FOO]
///     pub foo: bool,
///     #[property = BAR]
///     pub bar: u8,
/// }
/// ```
pub trait Properties: ReflectProperties + Copy + Eq + Hash + Default {
    /// Returns the block state definition.
    fn definition() -> &'static StateDefinition<Self>;

    /// Returns the property value corresponding to the index and downcast the value.
    ///
    /// Returns [`None`] if the block state does not contain the specified property.
    /// Or if the value type is not `T`.
    #[must_use]
    fn get_as<T: Value>(&self, index: Property<T>) -> Option<&T> {
        self.get(&index).and_then(<dyn ReflectValue>::downcast_ref)
    }

    /// Returns the property value corresponding to the index and downcast the value.
    ///
    /// Returns [`None`] if the block state does not contain the specified property.
    /// Or if the value type is not `T`.
    #[must_use]
    fn get_mut_as<T: Value>(&mut self, index: Property<T>) -> Option<&mut T> {
        self.get_mut(&index)
            .and_then(<dyn ReflectValue>::downcast_mut)
    }
}

/// The non-generic version of the [`Properties`] trait, can be made into trait object.
///
/// # Implementors
///
/// - `()` represnets the empty block state.
///
/// - struct with attrubute [`macro@property`].
#[downcast_sync]
pub trait ReflectProperties: AsAnySync + fmt::Debug + __SpecIndex {
    /// Tests for `self` and `other` values and types to be equal.
    fn dyn_eq(&self, other: &dyn ReflectProperties) -> bool;

    /// Feeds this value into the given [`hasher`](Hasher).
    fn dyn_hash(&self, hasher: &mut dyn Hasher);

    /// Clones the value and returns as the boxed trait object.
    fn dyn_clone(&self) -> Box<dyn ReflectProperties>;

    /// Downcasts the `other` value and applies it to the `self` value.
    ///
    /// Returns [`CastError`] if the inner type of `value` is not `Self`.
    fn dyn_clone_from(&mut self, other: &dyn ReflectProperties) -> Result<(), CastError>;

    /// Returns the non-generic block state definition.
    fn definition(&self) -> &'static dyn ReflectStateDefinition;

    /// Returns the non-generic block property value corresponding to the index.
    ///
    /// Returns [`None`] if the block state does not contain the specified property.
    fn get(&self, index: &dyn ReflectProperty) -> Option<&dyn ReflectValue>;

    /// Returns the non-generic block property value corresponding to the index.
    ///
    /// Returns [`None`] if the block state does not contain the specified property.
    fn get_mut(&mut self, index: &dyn ReflectProperty) -> Option<&mut dyn ReflectValue>;
}

impl dyn ReflectProperties {
    /// Returns the property value corresponding to the index and downcast the value.
    ///
    /// Returns [`None`] if the block state does not contain the specified property.
    /// Or if the value type is not `T`.
    #[must_use]
    pub fn get_as<T: Value>(&self, index: Property<T>) -> Option<&T> {
        self.get(&index).and_then(<dyn ReflectValue>::downcast_ref)
    }

    /// Returns the property value corresponding to the index and downcast the value.
    ///
    /// Returns [`None`] if the block state does not contain the specified property.
    /// Or if the value type is not `T`.
    #[must_use]
    pub fn get_mut_as<T: Value>(&mut self, index: Property<T>) -> Option<&mut T> {
        self.get_mut(&index)
            .and_then(<dyn ReflectValue>::downcast_mut)
    }
}

/// Definition of the block state. Provides mapping between state indexes and block states. Each
/// kind of block state shares with a same state definition instance.
///
/// The instance can be accessed by [`Properties::definition`] or [`ReflectProperties::definition`].
pub struct StateDefinition<T: Properties> {
    /// length of `.permutation` and `.mapping`.
    len: NonZeroU16,
    /// state index to the default block state.
    default: u16,
    /// maps state index to block state.
    permutation: Vec<T>,
    /// maps block state to state index.
    mapping: HashMap<T, u16>,
    /// maps property keys to property.
    keys: HashMap<&'static str, &'static dyn ReflectProperty>,
}

impl<T: Properties> StateDefinition<T> {
    #[must_use]
    #[doc(hidden)]
    pub fn __new(k: Vec<&'static dyn ReflectProperty>, v: Vec<T>) -> Self {
        let type_name = any::type_name::<T>();
        let len = v.len();
        let mut keys = HashMap::with_capacity(k.len());
        let mut mapping = HashMap::with_capacity(len);

        let Ok(len) = u16::try_from(len).and_then(NonZeroU16::try_from) else {
            use std::u16::MAX;
            panic!("block state `{type_name}` expects 1..{MAX} values, but got `{len}`");
        };

        for (key, property) in k.into_iter().map(|k| (k.key(), k)) {
            if keys.try_insert(key, property).is_err() {
                panic!("block state `{type_name}` has multiple properties with same name `{key}`");
            }
        }

        for (index, &value) in v.iter().enumerate() {
            if mapping.try_insert(value, index as u16).is_err() {
                panic!("block state `{type_name}` has multiple same values `{value:?}`");
            }
        }

        let default = <T as Default>::default();
        let Some(&default) = mapping.get(&default) else {
            panic!("block state `{type_name}` has invalid default value `{default:?}`");
        };

        Self {
            keys,
            len,
            default,
            mapping,
            permutation: v,
        }
    }

    /// Returns the block state corresponding to the state index.
    ///
    /// Returns [`None`] if the `index` is out of bounds (`>= self.len()`).
    #[must_use]
    pub fn get(&self, index: u16) -> Option<&T> {
        self.permutation.get(index as usize)
    }

    /// Returns the corresponding state index to the block state.
    ///
    /// Returns [`None`] if the `state` is not a valid block state.
    #[must_use]
    pub fn find(&self, state: &T) -> Option<u16> {
        self.mapping.get(state).copied()
    }
}

impl<T: Properties> fmt::Debug for StateDefinition<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = bevy::utils::get_short_name(any::type_name::<T>());
        write!(f, "StateDefinition<{type_name}>")?;
        f.debug_set().entries(self.keys.iter()).finish()
    }
}

/// The non-generic version of the [`StateDefinition<T>`], can be made into trait object.
///
/// # Implementors
///
/// The only implementor is [`StateDefinition<T>`].
// `self.len()` returns `NonZeroU16` and is never empty.
#[allow(clippy::len_without_is_empty)]
pub trait ReflectStateDefinition: AsAnySync + fmt::Debug {
    /// Returns the number of the possible block states.
    ///
    /// Guarantees that `self.len() > 0`.
    fn len(&self) -> NonZeroU16;

    /// Returns the state index corresponding to the default block state.
    ///
    /// Guarantees that `self.default() < self.len()`.
    fn default(&self) -> u16;

    /// Returns the non-generic block state corresponding to the state index.
    ///
    /// Returns [`None`] if the `index` is out of bounds (`>= self.len()`).
    fn dyn_get(&self, index: u16) -> Option<&dyn ReflectProperties>;

    /// Returns the corresponding state index to the non-generic block state.
    ///
    /// Returns [`None`] if the `state` is not a valid block state.
    fn dyn_find(&self, state: &dyn ReflectProperties) -> Option<u16>;
}

impl dyn ReflectStateDefinition {
    /// Returns the downcast value as &[`StateDefinition<T>`].
    ///
    /// Returns [`None`] if the inner type of `self` is not [`StateDefinition<T>`].
    #[must_use]
    pub fn downcast<T: Properties>(&self) -> Option<&StateDefinition<T>> {
        if self.type_id() == TypeId::of::<StateDefinition<T>>() {
            // SAFETY: the inner type of `other` is checked to be `Self`.
            Some(unsafe { &*(self as *const _ as *const StateDefinition<T>) })
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             PRIVATE                                            //
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Specialization of [`ReflectProperties::get_dyn`] and [`ReflectProperties::get_mut_dyn`].
#[doc(hidden)]
pub trait __SpecIndex {
    /// specialization of [`ReflectProperties::get_dyn`].
    fn spec_index(&self, index: &dyn ReflectProperty) -> Option<&dyn ReflectValue>;

    /// specialization of [`ReflectProperties::get_mut_dyn`].
    fn spec_index_mut(&mut self, index: &dyn ReflectProperty) -> Option<&mut dyn ReflectValue>;
}

/// Helper iterator to generate state permutation
#[doc(hidden)]
pub struct __StatePermutation<const N: usize> {
    idx: [usize; N],
    end: [usize; N],
    len: usize,
}

impl<const N: usize> __StatePermutation<N> {
    #[must_use]
    #[doc(hidden)]
    pub fn new(end: [usize; N]) -> Self {
        Self {
            idx: [0; N],
            end,
            len: end.into_iter().product(),
        }
    }
}

impl<const N: usize> Iterator for __StatePermutation<N> {
    type Item = [usize; N];

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.len -= 1;
            let item = self.idx;

            for i in 0..N {
                if self.idx[i] + 1 < self.end[i] {
                    self.idx[i] += 1;
                    break;
                } else {
                    self.idx[i] = 0;
                };
            }

            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<const N: usize> ExactSizeIterator for __StatePermutation<N> {}

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                      TRAIT IMPLEMENTATION                                      //
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Value> ReflectProperty for Property<T> {
    #[inline]
    fn key(&self) -> &'static str {
        self.key
    }

    #[inline]
    fn len(&self) -> NonZeroUsize {
        // SAFETY: `self.range.len() >= 2` is guaranteed when creation.
        unsafe { NonZeroUsize::new_unchecked(self.range.len()) }
    }

    fn dyn_eq(&self, other: &dyn ReflectProperty) -> bool {
        matches!(other.downcast(), Some(other) if other.eq(self))
    }

    fn dyn_iter(&self) -> Box<dyn Iterator<Item = &'static dyn ReflectValue>> {
        Box::new(self.iter().map(upcast_value))
    }

    fn dyn_contains(&self, value: &dyn ReflectValue) -> bool {
        matches!(value.downcast_ref(), Some(value) if self.contains(value))
    }

    fn dyn_cast(&self, value: ValueUntyped<'_>) -> Option<&'static dyn ReflectValue> {
        self.cast(value).map(upcast_value)
    }
}

impl<T: Properties> ReflectProperties for T {
    fn dyn_eq(&self, other: &dyn ReflectProperties) -> bool {
        matches!(other.downcast_ref(), Some(other) if self.eq(other))
    }

    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        self.hash(&mut hasher);
        self.type_id().hash(&mut hasher);
    }

    fn dyn_clone(&self) -> Box<dyn ReflectProperties> {
        Box::new(*self)
    }

    fn dyn_clone_from(&mut self, other: &dyn ReflectProperties) -> Result<(), CastError> {
        match other.downcast_ref() {
            #[allow(clippy::unit_arg)]
            Some(&other) => Ok(*self = other),
            None => Err(CastError {
                src: other.type_name(),
                dst: any::type_name::<T>(),
            }),
        }
    }

    fn definition(&self) -> &'static dyn ReflectStateDefinition {
        <T as Properties>::definition()
    }

    fn get(&self, index: &dyn ReflectProperty) -> Option<&dyn ReflectValue> {
        <T as __SpecIndex>::spec_index(self, index)
    }

    fn get_mut(&mut self, index: &dyn ReflectProperty) -> Option<&mut dyn ReflectValue> {
        <T as __SpecIndex>::spec_index_mut(self, index)
    }
}

impl<T: Properties> ReflectStateDefinition for StateDefinition<T> {
    #[inline]
    fn len(&self) -> NonZeroU16 {
        self.len
    }

    #[inline]
    fn default(&self) -> u16 {
        self.default
    }

    fn dyn_get(&self, index: u16) -> Option<&dyn ReflectProperties> {
        self.get(index).map(upcast_state)
    }

    fn dyn_find(&self, state: &dyn ReflectProperties) -> Option<u16> {
        self.find(state.downcast_ref()?)
    }
}

#[rustfmt::skip] #[inline(always)]
fn upcast_value<T: Value>(value: &T) -> &dyn ReflectValue { value }

#[rustfmt::skip] #[inline(always)]
fn upcast_state<T: Properties>(state: &T) -> &dyn ReflectProperties { state }
