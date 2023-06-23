use std::borrow::Cow;
use std::convert::Infallible;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::{any, fmt};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::dynamic::{downcast_sync, AsAnySync, CastError};
use crate::primitive;

// re-exports
pub use libcrium_core_macros::strenum;

/// Indicates that the value could convert between the enum type and the string literal.
///
/// The trait is the generic version of [`DynEnum`], see the [`documentation`](DynEnum) for more
/// details.
///
/// # Macros
///
/// The proc-macro attribute [`macro@strenum`] can declares enum type that is automatically
/// implemented the [`StrEnum`] and [`DynEnum`] trait.
pub trait StrEnum: DynEnum + Copy + Eq + Hash + FromStr<Err = Self::FromStrError> {
    /// [`Error`] that is [`<Self as FromStr>::Err`](FromStr::Err).
    type FromStrError: Error;

    /// A slice containing all the variants of the enum type.
    const VALUES: &'static [Self];
}

/// Indicates that the value could convert between the enum type and the string literal.
///
/// The trait is the non-generic version of [`StrEnum`], and can be made into trait object.
///
/// # Macros
///
/// The proc-macro attribute [`macro@strenum`] can declares enum type that is automatically
/// implemented the [`StrEnum`] and [`DynEnum`] trait.
pub trait DynEnum: AsAnySync + fmt::Debug + fmt::Display {
    /// Returns the enum value as a string literal.
    fn as_str(&self) -> &'static str;
}

/// A value that represents either a boolean, an integer or an enumeration.
///
/// Used to represents the value of the block property.
///
/// - [`ReflectValue`] is a super trait that can be made into trait object.
///
/// - [`ValueUntyped`] is the non-generic enum representation of the value.
///
/// # Implementors
///
/// - [`bool`] for boolean block property.
///
/// - [`u8`] for integer block property.
///
/// - enum implements [`StrEnum`] for enum block property.
pub trait Value: ReflectValue + Copy + Eq + Hash + FromStr<Err = Self::FromStrError> {
    /// [`Error`] that is [`<Self as FromStr>::Err`](FromStr::Err).
    type FromStrError: Error;

    /// [`Error`] that is returned by [`Value::from_value`].
    type FromValError: Error;

    /// Returns the value that is converted from the non-generic value.
    ///
    /// Returns an error if the conversion fails.
    fn from_value(value: ValueUntyped<'_>) -> Result<Self, Self::FromValError>;

    /// Converts the value into the non-generic version.
    ///
    /// NOTE: Never returns the [`Owned`](ValueUntyped::Owned) variant.
    fn into_value(self) -> ValueUntyped<'static>;
}

/// The non-generic version of the [`Value`] trait, can be made into trait object.
///
/// # Implementors
///
/// - [`bool`], for boolean block property.
///
/// - [`u8`], for integer block property.
///
/// - enum implements [`StrEnum`], for enum block property.
#[downcast_sync]
pub trait ReflectValue: AsAnySync {
    /// Tests for `self` and `other` values and types to be equal.
    fn dyn_eq(&self, other: &dyn ReflectValue) -> bool;

    /// Feeds this value into the given [`hasher`](Hasher).
    fn dyn_hash(&self, hasher: &mut dyn Hasher);

    /// Clones the value and returns as the boxed trait object.
    fn dyn_clone(&self) -> Box<dyn ReflectValue>;

    /// Downcasts the `other` value and applies it to the `self` value.
    ///
    /// Returns [`CastError`] if the inner type of `value` is not `Self`.
    fn dyn_clone_from(&mut self, other: &dyn ReflectValue) -> Result<(), CastError>;

    /// Parses the `other` value and applies it to the `self` value.
    ///
    /// Returns a boxed error if cannot parse the string.
    fn dyn_from_str(&mut self, other: &str) -> Result<(), Box<dyn Error>>;

    /// Downcasts the `other` value and applies it to the `self` value.
    ///
    /// Returns a boxed error if cannot cast `other` to `Self`.
    fn dyn_from_value(&mut self, other: ValueUntyped<'_>) -> Result<(), Box<dyn Error>>;

    /// Returns the value as the string literal.
    ///
    /// # Results
    ///
    /// - Returns `"true"` or `"false"` if `Self` is [`bool`].
    /// - Returns `"0"` ... `"255"` if `Self` is [`u8`].
    /// - Returns the specified string if `Self` implements [`StrEnum`].
    fn as_str(&self) -> &'static str;

    /// Returns the value as the non-generic version.
    ///
    /// # Results
    ///
    /// Returns [`Value::Boolean(_)`] if `Self` is [`bool`].
    /// Returns [`Value::Integer(_)`] if `Self` is [`u8`].
    /// Returns [`Value::Borrowed(_)`] if `Self` implements [`StrEnum`].
    fn untyped(&self) -> ValueUntyped<'static>;
}

impl Clone for Box<dyn ReflectValue> {
    #[inline]
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

impl ToOwned for dyn ReflectValue {
    type Owned = Box<dyn ReflectValue>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.dyn_clone()
    }
}

impl fmt::Debug for dyn ReflectValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for dyn ReflectValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Hash for dyn ReflectValue {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state)
    }
}

impl Eq for dyn ReflectValue {}

impl PartialEq for dyn ReflectValue {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl PartialEq<bool> for dyn ReflectValue {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        matches!(self.downcast_ref::<bool>(), Some(value) if other.eq(value))
    }
}

impl PartialEq<u8> for dyn ReflectValue {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        matches!(self.downcast_ref::<u8>(), Some(value) if other.eq(value))
    }
}

impl<T: StrEnum> PartialEq<T> for dyn ReflectValue {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        matches!(self.downcast_ref::<T>(), Some(value) if other.eq(value))
    }
}

impl PartialEq<ValueUntyped<'_>> for dyn ReflectValue {
    #[inline]
    fn eq(&self, other: &ValueUntyped<'_>) -> bool {
        self.untyped().eq(other)
    }
}

/// The enum representation of the block property value.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ValueUntyped<'s> {
    /// A boolean value of the block state property.
    Boolean(bool),

    /// An integral value of the block state property.
    Integer(u8),

    /// A string slice that represents the value of an enum property.
    Borrowed(&'s str),

    /// A boxed string that represents the value of an enum property.
    Owned(Box<str>),
}

impl<'s> ValueUntyped<'s> {
    /// Parses the given string to the untyped value.
    ///
    /// - If `s` is "true" or "false", returns [`Boolean(_)`](ValueUntyped::Boolean).
    ///
    /// - If `s` is "0" .. "255", returns [`Integer(_)`](ValueUntyped::Integer).
    ///
    /// - Otherwise, returns [`Borrowed(s)`](ValueUntyped::Borrowed).
    ///
    /// The method is similar to [`FromStr::from_str`], but borrowes the given string.
    #[must_use]
    pub fn from_borrowed_str(s: &'s str) -> Self {
        if let Ok(boolean) = <bool as FromStr>::from_str(s) {
            Self::Boolean(boolean)
        } else if let Ok(integer) = <u8 as FromStr>::from_str(s) {
            Self::Integer(integer)
        } else {
            Self::Borrowed(s)
        }
    }

    /// Returns the value that is converted from the non-generic value.
    ///
    /// Returns an error if the conversion fails.
    #[inline]
    pub fn typed<T: Value>(&self) -> Result<T, <T as Value>::FromValError> {
        <T as Value>::from_value(self.downgrade())
    }

    /// Upgrades the lifetime from `'s` to `'static`.
    ///
    /// If the value is [`ValueUntyped::Borrowed`], then boxes it to [`ValueUntyped::Owned`]. Never returns the
    /// [`ValueUntyped::Borrowed`] variant.
    #[must_use]
    pub fn upgrade(self) -> ValueUntyped<'static> {
        match self {
            Self::Boolean(boolean) => ValueUntyped::Boolean(boolean),
            Self::Integer(integer) => ValueUntyped::Integer(integer),
            Self::Borrowed(borrowed) => ValueUntyped::Owned(Box::from(borrowed)),
            Self::Owned(owned) => ValueUntyped::Owned(owned),
        }
    }

    /// Downgrades the lifetime from `'s` to '`_`'.
    ///
    /// If the value is [`ValueUntyped::Owned`], the borrowes it as [`ValueUntyped::Borrowed`]. Never returns the
    /// [`ValueUntyped::Owned`] variant.
    #[must_use]
    pub fn downgrade(&self) -> ValueUntyped<'_> {
        match self {
            Self::Boolean(boolean) => ValueUntyped::Boolean(*boolean),
            Self::Integer(integer) => ValueUntyped::Integer(*integer),
            Self::Borrowed(borrowed) => ValueUntyped::Borrowed(borrowed),
            Self::Owned(owned) => ValueUntyped::Borrowed(owned),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Boolean(boolean) => primitive::bool::to_str(*boolean),
            Self::Integer(integer) => primitive::u8::to_str(*integer),
            Self::Borrowed(borrowed) => borrowed,
            Self::Owned(owned) => owned,
        }
    }

    /// Returns the boolean variant of the value. Or returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libcrium_core::strenum::ValueUntyped;
    /// #
    /// let b = ValueUntyped::from(false);
    /// let i = ValueUntyped::from(23);
    /// let s = ValueUntyped::from("hello, world");
    ///
    /// assert_eq!(b.bool_or_none(), Some(false));
    /// assert_eq!(i.bool_or_none(), None);
    /// assert_eq!(s.bool_or_none(), None);
    /// ```
    #[must_use]
    pub fn bool_or_none(&self) -> Option<bool> {
        if let Self::Boolean(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    /// Returns the integer variant of the value. Or returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libcrium_core::strenum::ValueUntyped;
    /// #
    /// let b = ValueUntyped::from(false);
    /// let i = ValueUntyped::from(23);
    /// let s = ValueUntyped::from("hello, world");
    ///
    /// assert_eq!(b.int_or_none(), None);
    /// assert_eq!(i.int_or_none(), Some(23));
    /// assert_eq!(s.int_or_none(), None);
    /// ```
    #[must_use]
    pub fn int_or_none(&self) -> Option<u8> {
        if let Self::Integer(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    /// Returns the string variant of the value. Or returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use libcrium_core::strenum::ValueUntyped;
    /// #
    /// let b = ValueUntyped::from(false);
    /// let i = ValueUntyped::from(23);
    /// let s = ValueUntyped::from("hello, world");
    ///
    /// assert_eq!(b.str_or_none(), None);
    /// assert_eq!(i.str_or_none(), None);
    /// assert_eq!(s.str_or_none(), Some("hello, world"));
    /// ```
    #[must_use]
    pub fn str_or_none(&self) -> Option<&str> {
        match self {
            Self::Borrowed(borrowed) => Some(borrowed),
            Self::Owned(owned) => Some(owned),
            _ => None,
        }
    }

    /// Applies the function `f` to the boolean variant.
    ///
    /// Returns [`None`] if the value is not a boolean value.
    #[must_use]
    pub fn bool_and_then<R, F: FnOnce(bool) -> R>(&self, f: F) -> Option<R> {
        if let Self::Boolean(boolean) = self {
            Some(f(*boolean))
        } else {
            None
        }
    }

    /// Applies the function `f` to the integer variant.
    ///
    /// Returns [`None`] if the value is not an integer value.
    #[must_use]
    pub fn int_and_then<R, F: FnOnce(u8) -> R>(&self, f: F) -> Option<R> {
        if let Self::Integer(integer) = self {
            Some(f(*integer))
        } else {
            None
        }
    }

    /// Applies the function `f` to the string variant.
    ///
    /// Returns [`None`] if the value is not a string value.
    #[must_use]
    pub fn str_and_then<R, F: FnOnce(&str) -> R>(&self, f: F) -> Option<R> {
        match self {
            Self::Borrowed(borrowed) => Some(f(borrowed)),
            Self::Owned(owned) => Some(f(owned)),
            _ => None,
        }
    }

    /// Returns the boolean variant of the value.
    ///
    /// Or returns the result of function `f` as an error.
    pub fn bool_or_else<R, F: FnOnce() -> R>(&self, f: F) -> Result<bool, R> {
        if let Self::Boolean(boolean) = self {
            Ok(*boolean)
        } else {
            Err(f())
        }
    }

    /// Returns the integer variant of the value.
    ///
    /// Or returns the result of function `f` as an error.
    pub fn int_or_else<R, F: FnOnce() -> R>(&self, f: F) -> Result<u8, R> {
        if let Self::Integer(integer) = self {
            Ok(*integer)
        } else {
            Err(f())
        }
    }

    /// Returns the string variant of the value.
    ///
    /// Or returns the result of function `f` as an error.
    pub fn str_or_else<R, F: FnOnce() -> R>(&self, f: F) -> Result<&str, R> {
        match self {
            Self::Borrowed(borrowed) => Ok(borrowed),
            Self::Owned(owned) => Ok(owned),
            _ => Err(f()),
        }
    }
}

impl<'s> AsRef<str> for ValueUntyped<'s> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'s> fmt::Debug for ValueUntyped<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ValueUntyped(")?;
        f.write_str(self.as_str())?;
        f.write_str(")")
    }
}

impl<'s> fmt::Display for ValueUntyped<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'s> FromStr for ValueUntyped<'s> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(boolean) = <bool as FromStr>::from_str(s) {
            Ok(Self::Boolean(boolean))
        } else if let Ok(integer) = <u8 as FromStr>::from_str(s) {
            Ok(Self::Integer(integer))
        } else {
            Ok(Self::Owned(Box::from(s)))
        }
    }
}

impl<'s> Serialize for ValueUntyped<'s> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ValueUntyped::Boolean(boolean) => <bool as Serialize>::serialize(boolean, serializer),
            ValueUntyped::Integer(integer) => <u8 as Serialize>::serialize(integer, serializer),
            ValueUntyped::Borrowed(borrowed) => <str as Serialize>::serialize(borrowed, serializer),
            ValueUntyped::Owned(owned) => <str as Serialize>::serialize(owned, serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ValueUntyped<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = ValueUntyped<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("bool, u8 or string")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(ValueUntyped::Boolean(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                match <u8 as TryFrom<_>>::try_from(v) {
                    Ok(integer) => Ok(ValueUntyped::Integer(integer)),
                    Err(err) => Err(E::custom(err)),
                }
            }

            fn visit_i128<E: de::Error>(self, v: i128) -> Result<Self::Value, E> {
                match <u8 as TryFrom<_>>::try_from(v) {
                    Ok(integer) => Ok(ValueUntyped::Integer(integer)),
                    Err(err) => Err(E::custom(err)),
                }
            }

            fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E> {
                Ok(ValueUntyped::Integer(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                match <u8 as TryFrom<_>>::try_from(v) {
                    Ok(integer) => Ok(ValueUntyped::Integer(integer)),
                    Err(err) => Err(E::custom(err)),
                }
            }

            fn visit_u128<E: de::Error>(self, v: u128) -> Result<Self::Value, E> {
                match <u8 as TryFrom<_>>::try_from(v) {
                    Ok(integer) => Ok(ValueUntyped::Integer(integer)),
                    Err(err) => Err(E::custom(err)),
                }
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(ValueUntyped::Owned(Box::from(v)))
            }

            fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
                Ok(ValueUntyped::Borrowed(v))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(ValueUntyped::Owned(v.into_boxed_str()))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl<'s> From<bool> for ValueUntyped<'s> {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl<'s> From<u8> for ValueUntyped<'s> {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Integer(value)
    }
}

impl<'s> From<&'s str> for ValueUntyped<'s> {
    #[inline]
    fn from(value: &'s str) -> Self {
        Self::Borrowed(value)
    }
}

impl<'s> From<Box<str>> for ValueUntyped<'s> {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self::Owned(value)
    }
}

impl<'s> From<String> for ValueUntyped<'s> {
    #[inline]
    fn from(value: String) -> Self {
        Self::Owned(value.into_boxed_str())
    }
}

impl<'s> From<Cow<'s, str>> for ValueUntyped<'s> {
    #[inline]
    fn from(value: Cow<'s, str>) -> Self {
        match value {
            Cow::Borrowed(borrowed) => Self::Borrowed(borrowed),
            Cow::Owned(owned) => Self::Owned(owned.into_boxed_str()),
        }
    }
}

impl<'s, T: StrEnum> From<T> for ValueUntyped<'s> {
    #[inline]
    fn from(value: T) -> Self {
        ValueUntyped::Borrowed(value.as_str())
    }
}

impl<'s> TryFrom<ValueUntyped<'s>> for bool {
    type Error = ValueToBooleanError;

    #[inline]
    fn try_from(value: ValueUntyped<'s>) -> Result<Self, Self::Error> {
        if let ValueUntyped::Boolean(boolean) = value {
            Ok(boolean)
        } else {
            Err(ValueToBooleanError)
        }
    }
}

impl<'s> TryFrom<ValueUntyped<'s>> for u8 {
    type Error = ValueToIntegerError;

    #[inline]
    fn try_from(value: ValueUntyped<'s>) -> Result<Self, Self::Error> {
        if let ValueUntyped::Integer(integer) = value {
            Ok(integer)
        } else {
            Err(ValueToIntegerError)
        }
    }
}

impl<'s> TryFrom<ValueUntyped<'s>> for Cow<'s, str> {
    type Error = ValueToStringError;

    #[inline]
    fn try_from(value: ValueUntyped<'s>) -> Result<Self, Self::Error> {
        match value {
            ValueUntyped::Borrowed(borrowed) => Ok(Cow::Borrowed(borrowed)),
            ValueUntyped::Owned(owned) => Ok(Cow::Owned(owned.into_string())),
            _ => Err(ValueToStringError),
        }
    }
}

impl<'s> PartialEq<bool> for ValueUntyped<'s> {
    fn eq(&self, other: &bool) -> bool {
        matches!(self, Self::Boolean(value) if value == other)
    }
}

impl<'s> PartialEq<u8> for ValueUntyped<'s> {
    fn eq(&self, other: &u8) -> bool {
        matches!(self, Self::Integer(value) if value == other)
    }
}

impl<'s, T: StrEnum> PartialEq<T> for ValueUntyped<'s> {
    fn eq(&self, other: &T) -> bool {
        match self {
            Self::Borrowed(borrowed) => <str as PartialEq>::eq(borrowed, other.as_str()),
            Self::Owned(owned) => <str as PartialEq>::eq(owned, other.as_str()),
            _ => false,
        }
    }
}

impl<'s> PartialEq<dyn ReflectValue> for ValueUntyped<'s> {
    #[inline]
    fn eq(&self, other: &dyn ReflectValue) -> bool {
        other.untyped().eq(self)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             ERRORS                                             //
////////////////////////////////////////////////////////////////////////////////////////////////////

/// An [`error`](Error) that is [`<bool as TryFrom<ValueUntyped<'s>>>::Error`](TryFrom::Error).
#[derive(Clone, Copy, Debug, Default)]
pub struct ValueToBooleanError;

impl Error for ValueToBooleanError {}

impl fmt::Display for ValueToBooleanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expects a boolean value")
    }
}

/// An [`error`](Error) that is [`<u8 as TryFrom<ValueUntyped<'s>>>::Error`](TryFrom::Error).
#[derive(Clone, Copy, Debug, Default)]
pub struct ValueToIntegerError;

impl Error for ValueToIntegerError {}

impl fmt::Display for ValueToIntegerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expects an integral value")
    }
}

/// An [`error`](Error) that is [`<Cow<'s, str> as TryFrom<ValueUntyped<'s>>>::Error`](TryFrom::Error).
#[derive(Clone, Copy, Debug, Default)]
pub struct ValueToStringError;

impl Error for ValueToStringError {}

impl fmt::Display for ValueToStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expects a string value")
    }
}

/// An [`error`](Error) that is [`<T as Value>::FromValError`](Value::FromValError) where
/// `T` implements [`StrEnum`].
pub enum ValueToEnumError<T: StrEnum> {
    /// Error returned from [`<Cow<str> as TryFrom<ValueUntyped<'_>>>::try_from()`](TryFrom).
    TryFromError(ValueToStringError),
    /// Error returned from [`<T as FromStr>::from_str()`](FromStr)
    FromStrError(<T as FromStr>::Err),
}

impl<T: StrEnum> Error for ValueToEnumError<T> {}

impl<T: StrEnum> fmt::Debug for ValueToEnumError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromError(err) => <_ as fmt::Debug>::fmt(err, f),
            Self::FromStrError(err) => <_ as fmt::Debug>::fmt(err, f),
        }
    }
}

impl<T: StrEnum> fmt::Display for ValueToEnumError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromError(err) => <_ as fmt::Display>::fmt(err, f),
            Self::FromStrError(err) => <_ as fmt::Display>::fmt(err, f),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                      TRAIT IMPLEMENTATION                                      //
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Value for bool {
    type FromStrError = <Self as FromStr>::Err;
    type FromValError = ValueToBooleanError;

    #[inline]
    fn from_value(value: ValueUntyped<'_>) -> Result<Self, Self::FromValError> {
        <Self as TryFrom<_>>::try_from(value)
    }

    #[inline]
    fn into_value(self) -> ValueUntyped<'static> {
        ValueUntyped::Boolean(self)
    }
}

impl Value for u8 {
    type FromStrError = <Self as FromStr>::Err;
    type FromValError = ValueToIntegerError;

    #[inline]
    fn from_value(value: ValueUntyped<'_>) -> Result<Self, Self::FromValError> {
        <Self as TryFrom<_>>::try_from(value)
    }

    #[inline]
    fn into_value(self) -> ValueUntyped<'static> {
        ValueUntyped::Integer(self)
    }
}

impl<T: StrEnum> Value for T {
    type FromStrError = <Self as FromStr>::Err;
    type FromValError = ValueToEnumError<T>;

    fn from_value(value: ValueUntyped<'_>) -> Result<Self, Self::FromValError> {
        match <Cow<str> as TryFrom<_>>::try_from(value) {
            Ok(value) => match <Self as FromStr>::from_str(&value) {
                Ok(value) => Ok(value),
                Err(err) => Err(ValueToEnumError::FromStrError(err)),
            },
            Err(err) => Err(ValueToEnumError::TryFromError(err)),
        }
    }

    #[inline]
    fn into_value(self) -> ValueUntyped<'static> {
        ValueUntyped::Borrowed(<T as DynEnum>::as_str(&self))
    }
}

impl<T: Value> ReflectValue for T {
    fn dyn_eq(&self, other: &dyn ReflectValue) -> bool {
        matches!(other.downcast_ref::<T>(), Some(other) if self.eq(other))
    }

    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        self.hash(&mut hasher);
        self.type_id().hash(&mut hasher);
    }

    fn dyn_clone(&self) -> Box<dyn ReflectValue> {
        Box::new(*self)
    }

    fn dyn_clone_from(&mut self, other: &dyn ReflectValue) -> Result<(), CastError> {
        match other.downcast_ref() {
            #[allow(clippy::unit_arg)]
            Some(&other) => Ok(*self = other),
            None => Err(CastError {
                src: other.type_name(),
                dst: any::type_name::<T>(),
            }),
        }
    }

    fn dyn_from_str(&mut self, other: &str) -> Result<(), Box<dyn Error>> {
        match <Self as FromStr>::from_str(other) {
            #[allow(clippy::unit_arg)]
            Ok(other) => Ok(*self = other),
            Err(error) => Err(Box::new(error)),
        }
    }

    fn dyn_from_value(&mut self, other: ValueUntyped<'_>) -> Result<(), Box<dyn Error>> {
        match <Self as Value>::from_value(other) {
            #[allow(clippy::unit_arg)]
            Ok(other) => Ok(*self = other),
            Err(error) => Err(Box::new(error)),
        }
    }

    fn as_str(&self) -> &'static str {
        match <T as Value>::into_value(*self) {
            ValueUntyped::Boolean(boolean) => primitive::bool::to_str(boolean),
            ValueUntyped::Integer(integer) => primitive::u8::to_str(integer),
            ValueUntyped::Borrowed(borrowed) => borrowed,
            // SAFETY: `Self::into_value()` guarantees that never returns `Owned(_)`.
            ValueUntyped::Owned(_) => unreachable!(),
        }
    }

    fn untyped(&self) -> ValueUntyped<'static> {
        <T as Value>::into_value(*self)
    }
}
