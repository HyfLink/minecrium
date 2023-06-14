use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

use crate::dynamic::AsAnySync;

/// Indicates that the value could convert between the enum type and the string literal.
///
/// The trait is the generic version of [`DynEnum`], see the [`documentation`](DynEnum) for more
/// details.
pub trait StrEnum: DynEnum + Copy + Eq + Hash + FromStr {
    /// A slice containing all the variants of the enum type.
    const VALUES: &'static [Self];
}

/// Indicates that the value could convert between the enum type and the string literal.
///
/// The trait is the non-generic version of [`StrEnum`], and can be made into trait object.
pub trait DynEnum: AsAnySync + fmt::Debug + fmt::Display {
    /// Returns the enum value as a string literal.
    fn as_str(&self) -> &'static str;
}


