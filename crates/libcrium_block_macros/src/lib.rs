//! the proc-macro crate of the `libcrium_block` crate.

use proc_macro::TokenStream as StdTokenStream;

mod property;

/// Implements the `Properties` trait for the derive input.
///
/// # Examples
///
/// ```ignore
/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// #[property(crate = libcrium_block)]
/// pub struct FooBar {
///     #[property = FOO]
///     foo: bool,
///     #[property = BAR]
///     pub bar: u8,
/// }
/// ```
/// 1. `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]`
///
///    The 7 traits are required by the `Properites`.
///
/// 1. `#[property(crate = $PATH)]` (optional)
///
///    Specifies the `libcrium_block` crate path. If missing, uses `libcrium_block`.
///
/// 1. `#[property = $PROPERTY]` (required)
///
///    Specifies the block property definition.
///    For example, `foo: bool` corresponds to `FOO: Property<bool> `.
#[proc_macro_attribute]
pub fn property(attrs: StdTokenStream, input: StdTokenStream) -> StdTokenStream {
    let attrs = syn::parse_macro_input!(attrs as property::MacroAttrs);
    let input = syn::parse_macro_input!(input as property::MacroInput);
    StdTokenStream::from(match property::expand_property(attrs, input) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    })
}
