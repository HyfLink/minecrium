use proc_macro::TokenStream;

mod downcast;

/// Implements following methods for the trait object.
///
/// ```rust, ignore
/// impl dyn TRAIT {
///     fn is<T: TRAIT>(&self) -> bool;
///     fn downcast_ref<T: TRAIT>(&self) -> Option<&T>;
///     fn downcast_mut<T: TRAIT>(&mut self) -> Option<&mut T>;
///     fn downcast_rc<T: TRAIT>(self: Rc<Self>) -> Result<Rc<T>, Rc<dyn Any>>;
///     fn downcast_boxed<T: TRAIT>(self: Box<Self>) -> Result<Box<T>, Box<dyn Any>>;
/// }
/// ```
#[proc_macro_attribute]
pub fn downcast(_: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(input as syn::ItemTrait);
    TokenStream::from(downcast::proc_macro_downcast(item))
}

/// Implements following methods for the trait object.
///
/// ```rust, ignore
/// impl dyn TRAIT {
///     fn is<T: TRAIT>(&self) -> bool;
///     fn downcast_ref<T: TRAIT>(&self) -> Option<&T>;
///     fn downcast_mut<T: TRAIT>(&mut self) -> Option<&mut T>;
///     fn downcast_rc<T: TRAIT>(self: Rc<Self>) -> Result<Rc<T>, Rc<dyn Any>>;
///     fn downcast_arc<T: TRAIT>(self: Arc<Self>) -> Result<Arc<T>, Arc<dyn Any + Send + Sync>>;
///     fn downcast_boxed<T: TRAIT>(self: Box<Self>) -> Result<Box<T>, Box<dyn Any + Send + Sync>>;
/// }
/// ```
#[proc_macro_attribute]
pub fn downcast_sync(_: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(input as syn::ItemTrait);
    TokenStream::from(downcast::proc_macro_downcast_sync(item))
}
