use std::any::{self, TypeId};
use std::hash::Hash;
use std::ops::Range;
use std::{fmt, num::NonZeroUsize};

use libcrium_core::dynamic::{AsAnySync, CastError};
use libcrium_core::primitive;
use libcrium_core::strenum::StrEnum;

// re-exports
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
    /// Returns the block property as the non-generic version.
    #[must_use]
    pub fn untyped(&self) -> PropertyUntyped<'_> {
        PropertyUntyped { inner: self }
    }

    /// Returns the unique key of the block property.
    #[must_use]
    pub fn key(&self) -> &'static str {
        self.key
    }

    /// Returns the number of elements in the property.
    #[must_use]
    pub fn len(&self) -> NonZeroUsize {
        // SAFETY: `self.range.len() >= 2` is guaranteed when creation.
        unsafe { NonZeroUsize::new_unchecked(self.range.len()) }
    }

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
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
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

/// A reference to the non-generic version of [`Property<T>`].
#[derive(Clone, Copy)]
pub struct PropertyUntyped<'a> {
    inner: &'a dyn ReflectProperty,
}

impl<'a> PropertyUntyped<'a> {
    /// Returns the downcast block property as [`Property<T>`].
    ///
    /// Returns [`CastError`] if the property value type is not `T`.
    ///
    /// This method is equivalent to [`Property<T>`] as [`TryFrom<PropertyUntyped<'_>>`].
    pub fn typed<T: Value>(self) -> Result<Property<T>, CastError> {
        if self.inner.type_id() == TypeId::of::<Property<T>>() {
            // SAFETY: the inner type of `self.inner` is checked to be `Property<T>`.
            Ok(unsafe { *(self.inner as *const _ as *const Property<T>) })
        } else {
            Err(CastError {
                src: self.inner.type_name(),
                dst: any::type_name::<Property<T>>(),
            })
        }
    }

    /// Returns the unique key of the block property.
    #[inline]
    #[must_use]
    pub fn key(self) -> &'static str {
        self.inner.key()
    }

    /// Returns the number of elements in the property.
    #[inline]
    #[must_use]
    pub fn len(self) -> NonZeroUsize {
        self.inner.len()
    }

    /// Returns an iterator over all the elements of the property.
    #[inline]
    pub fn iter(self) -> Box<dyn Iterator<Item = &'static dyn ReflectValue>> {
        self.inner.iter()
    }

    /// Returns `true` if the block property contains the `value`.
    #[inline]
    #[must_use]
    pub fn contains(self, value: &dyn ReflectValue) -> bool {
        self.inner.contains(value)
    }

    /// Returns the value that is converted from the non-generic value.
    ///
    /// Returns [`None`] if the conversion failds or if the block property does not contain the
    /// value.
    #[inline]
    #[must_use]
    pub fn cast(self, value: ValueUntyped<'_>) -> Option<&'static dyn ReflectValue> {
        self.inner.cast(value)
    }
}

impl<'a> fmt::Debug for PropertyUntyped<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <dyn ReflectProperty as fmt::Debug>::fmt(self.inner, f)
    }
}

impl<'a> Hash for PropertyUntyped<'a> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl<'a> Eq for PropertyUntyped<'a> {}

impl<'a> PartialEq for PropertyUntyped<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.type_id() == other.inner.type_id() && self.inner.key() == other.inner.key()
    }
}

impl<'a, T: Value> PartialEq<Property<T>> for PropertyUntyped<'a> {
    #[inline]
    fn eq(&self, other: &Property<T>) -> bool {
        matches!(self.typed::<T>(), Ok(this) if this.eq(other))
    }
}

impl<'a, T: Value> PartialEq<PropertyUntyped<'a>> for Property<T> {
    #[inline]
    fn eq(&self, other: &PropertyUntyped<'a>) -> bool {
        matches!(other.typed::<T>(), Ok(other) if other.eq(self))
    }
}

impl<'a, T: Value> TryFrom<PropertyUntyped<'a>> for Property<T> {
    type Error = CastError;

    #[inline]
    fn try_from(value: PropertyUntyped<'a>) -> Result<Self, Self::Error> {
        value.typed()
    }
}

/// Helper trait to implement [`PropertyUntyped`].
trait ReflectProperty: AsAnySync + fmt::Debug {
    /// Returns the unique key of the block property.
    fn key(&self) -> &'static str;

    /// Returns the number of elements in the property.
    fn len(&self) -> NonZeroUsize;

    /// Returns an iterator over all the elements of the property.
    fn iter(&self) -> Box<dyn Iterator<Item = &'static dyn ReflectValue>>;

    /// Returns `true` if the block property contains the `value`.
    fn contains(&self, value: &dyn ReflectValue) -> bool;

    /// Returns the value that is converted from the non-generic value.
    fn cast(&self, value: ValueUntyped<'_>) -> Option<&'static dyn ReflectValue>;
}

impl<T: Value> ReflectProperty for Property<T> {
    fn key(&self) -> &'static str {
        Property::key(self)
    }

    fn len(&self) -> NonZeroUsize {
        Property::len(self)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &'static dyn ReflectValue>> {
        Box::new(Property::iter(self).map(|value| value as _))
    }

    fn contains(&self, value: &dyn ReflectValue) -> bool {
        matches!(value.downcast_ref(), Some(value) if Property::contains(self, value))
    }

    fn cast(&self, value: ValueUntyped<'_>) -> Option<&'static dyn ReflectValue> {
        Property::cast(self, value).map(|value| value as _)
    }
}
