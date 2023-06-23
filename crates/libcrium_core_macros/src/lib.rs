//! the proc-macro crate of the `libcrium_core` crate.

use proc_macro::TokenStream;

mod downcast;
mod strenum;

/// Implements downcast methods for `dyn TRAIT`.
///
/// Requires the trait to have a super trait `AsAny`, which is defined in the `libcrium_core`
/// crate.
///
/// ```ignore
/// impl dyn TRAIT {
///     /// Returns `true` if the inner type is the same as `T`.
///     pub fn is<T: TRAIT>(&self) -> bool;
///     
///     /// Returns the downcast value as `&T`.
///     pub fn downcast_ref<T: TRAIT>(&self) -> Option<&T>;
///
///     /// Returns the downcast value as `&mut T`.
///     pub fn downcast_mut<T: TRAIT>(&mut self) -> Option<&mut T>;
///
///     /// Returns the downcast value as `Box<T>`.
///     pub fn downcast<T: TRAIT>(self: Box<Self>) -> Result<Box<T>, Box<dyn TRAIT>>;
///
///     /// Returns the downcast value as `Rc<T>`.
///     pub fn downcast_rc<T: TRAIT>(self: Rc<Self>) -> Result<Rc<T>, Rc<dyn TRAIT>>;
/// }
/// ```
///
/// # Attributes
///
/// The proc-macro attribute ignores any meta.
#[proc_macro_attribute]
pub fn downcast(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemTrait);
    TokenStream::from(downcast::expand_downcast(input))
}

/// Implements downcast methods for `dyn TRAIT`.
///
/// Requires the trait to have a super trait `AsAnySync`, which is defined in the `libcrium_core`
/// crate.
///
/// ```ignore
/// impl dyn TRAIT {
///     /// Returns `true` if the inner type is the same as `T`.
///     pub fn is<T: TRAIT>(&self) -> bool;
///     
///     /// Returns the downcast value as `&T`.
///     pub fn downcast_ref<T: TRAIT>(&self) -> Option<&T>;
///
///     /// Returns the downcast value as `&mut T`.
///     pub fn downcast_mut<T: TRAIT>(&mut self) -> Option<&mut T>;
///
///     /// Returns the downcast value as `Box<T>`.
///     pub fn downcast<T: TRAIT>(self: Box<Self>) -> Result<Box<T>, Box<dyn TRAIT>>;
///
///     /// Returns the downcast value as `Rc<T>`.
///     pub fn downcast_rc<T: TRAIT>(self: Rc<Self>) -> Result<Rc<T>, Rc<dyn TRAIT>>;
///
///     /// Returns the downcast value as `Arc<T>`.
///     pub fn downcast_arc<T: TRAIT>(self: Arc<Self>) -> Result<Arc<T>, Arc<dyn TRAIT>>;
/// }
/// ```
///
/// # Attributes
///
/// The proc-macro attribute ignores any meta.
#[proc_macro_attribute]
pub fn downcast_sync(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemTrait);
    TokenStream::from(downcast::expand_downcast_sync(input))
}

/// Declares an `enum` type, and implements following traits:
///
/// - [`Clone`]
/// - [`Copy`]
/// - [`PartialEq`]
/// - [`Eq`]
/// - [`Hash`](std::hash::Hash)
/// - [`Debug`](std::fmt::Debug)
/// - [`Display`](std::fmt::Display)
/// - [`FromStr`](std::str::FromStr)
/// - `StrEnum` (defined in `libcrium_core`)
/// - `DynEnum` (defined in `libcrium_core`)
///
/// # Format
///
/// ```text
/// #[ ... ]
/// pub enum EnumType {
///     Variant0 = "variant0",
///     Variant1 = "variant1",
///     ...
/// }
/// ```
///
/// # Attributes
///
/// - `#[strenum(crate = $CRATE)]` (optional)
///
///   Specifies the `libcrium_core` crate path. If missing, uses `libcrium_core`.
///
/// - `#[strenum(error = $ERROR)]` (optional)
///
///   Specifies the [`FromStr::Err`](std::str::FromStr::Err) of the enum type. If missing, generates
///   an error named `{ENUM}FromStrError` where `{ENUM}` is the name of the enum type.
#[proc_macro_attribute]
pub fn strenum(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = syn::parse_macro_input!(attrs as strenum::StrEnumAttrs);
    let input = syn::parse_macro_input!(input as strenum::StrEnumInput);
    TokenStream::from(strenum::expand_strenum(attrs, input))
}
