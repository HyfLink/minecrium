use proc_macro::TokenStream;

mod downcast;

/// Implements methods for `dyn TRAIT`.
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
/// # Formats
///
/// - `#[downcast]`
/// - `#[downcast(path::to::minecrium_common)]`
#[proc_macro_attribute]
pub fn downcast(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = syn::parse_macro_input!(attrs as downcast::AttributeArgs);
    let input = syn::parse_macro_input!(input as syn::ItemTrait);
    TokenStream::from(downcast::proc_macro_downcast(attrs, input))
}

/// Implements methods for `dyn TRAIT`.
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

///     /// Returns the downcast value as `Arc<T>`.
///     pub fn downcast_arc<T: TRAIT>(self: Arc<Self>) -> Result<Arc<T>, Arc<dyn TRAIT>>;
/// }
/// ```
///
/// # Formats
///
/// - `#[downcast_sync]`
/// - `#[downcast_sync(path::to::minecrium_common)]`
#[proc_macro_attribute]
pub fn downcast_sync(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = syn::parse_macro_input!(attrs as downcast::AttributeArgs);
    let input = syn::parse_macro_input!(input as syn::ItemTrait);
    TokenStream::from(downcast::proc_macro_downcast_sync(attrs, input))
}
