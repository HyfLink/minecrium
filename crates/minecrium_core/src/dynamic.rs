use std::any::{self, Any};
use std::rc::Rc;
use std::sync::Arc;

pub use minecrium_core_macros::{downcast, downcast_sync};

/// Provides upcasts from the object to [`dyn Any`](Any).
///
/// This trait is automatically implemented for types that implemenet the
/// [`Any`] trait.
///
/// # Examples
///
/// ```no_run
/// # use minecrium_core::dynamic::{downcast, AsAny};
/// #
/// #[downcast]
/// trait DynEq: AsAny {
///     fn dyn_eq(&self, other: &dyn DynEq) -> bool;
///     fn dyn_ne(&self, other: &dyn DynEq) -> bool {
///         !self.dyn_eq(other)
///     }
/// }
///
/// impl<T: AsAny + Eq> DynEq for T {
///     fn dyn_eq(&self, other: &dyn DynEq) -> bool {
///         match other.downcast_ref() {
///             Some(other) => self.eq(other),
///             None => false,
///         }
///     }
/// }
/// ```
pub trait AsAny: Any {
    /// Returns the name of a type as a string slice.
    ///
    /// See [`type name`](any::type_name).
    fn type_name(&self) -> &'static str;

    /// Returns the value as [`&dyn Any`](Any).
    fn as_any(&self) -> &dyn Any;

    /// Returns the value as [`&mut dyn Any`](Any).
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns the value as [`Box<dyn Any>`].
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Returns the value as [`Rc<dyn Any>`].
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
}

impl<T: Any> AsAny for T {
    #[rustfmt::skip] #[inline]
    fn type_name(&self) -> &'static str { any::type_name::<Self>() }

    #[rustfmt::skip] #[inline]
    fn as_any(&self) -> &dyn Any { self }

    #[rustfmt::skip] #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any { self }

    #[rustfmt::skip] #[inline]
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }

    #[rustfmt::skip] #[inline]
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any> { self }
}

/// Provides upcasts from the object to [`dyn Any + Send + Sync`](Any).
///
/// This trait is automatically implemented for types that implemenet the
/// [`Any`], [`Send`], [`Sync`] traits.
///
/// # Examples
///
/// ```no_run
/// # use minecrium_core::dynamic::{downcast, AsAnySync};
/// #
/// #[downcast]
/// trait DynEq: AsAnySync {
///     fn dyn_eq(&self, other: &dyn DynEq) -> bool;
///     fn dyn_ne(&self, other: &dyn DynEq) -> bool {
///         !self.dyn_eq(other)
///     }
/// }
///
/// impl<T: AsAnySync + Eq> DynEq for T {
///     fn dyn_eq(&self, other: &dyn DynEq) -> bool {
///         match other.downcast_ref() {
///             Some(other) => self.eq(other),
///             None => false,
///         }
///     }
/// }
/// ```
pub trait AsAnySync: AsAny + Send + Sync {
    /// Returns the value as [`&(dyn Any + Send + Sync)`](Any).
    fn as_any_sync(&self) -> &(dyn Any + Send + Sync);

    /// Returns the value as [`&mut (dyn Any + Send + Sync)`](Any).
    fn as_any_sync_mut(&mut self) -> &mut (dyn Any + Send + Sync);

    /// Returns the value as [`Box<dyn Any + Send + Sync>`].
    fn into_any_sync(self: Box<Self>) -> Box<dyn Any + Send + Sync>;

    /// Returns the value as [`Arc<dyn Any + Send + Sync>`].
    fn into_any_sync_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<T: Any + Send + Sync> AsAnySync for T {
    #[rustfmt::skip] #[inline]
    fn as_any_sync(&self) -> &(dyn Any + Send + Sync) { self }

    #[rustfmt::skip] #[inline]
    fn as_any_sync_mut(&mut self) -> &mut (dyn Any + Send + Sync) { self }

    #[rustfmt::skip] #[inline]
    fn into_any_sync(self: Box<Self>) -> Box<dyn Any + Send + Sync> { self }

    #[rustfmt::skip] #[inline]
    fn into_any_sync_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> { self }
}
