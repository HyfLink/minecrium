use std::any::{self, Any};
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

/// Upcasts the trait object to [`dyn Any`](Any).
///
/// The trait is automatically implemented for any `'static` type that implemenets the
/// [`Any`] trait.
pub trait AsAny: Any {
    /// Returns the [`type name`](any::type_name) as a string slice.
    #[must_use]
    fn type_name(&self) -> &'static str;

    /// Returns the value as [`&dyn Any`](Any).
    #[must_use]
    fn as_any(&self) -> &dyn Any;

    /// Returns the value as [`&mut dyn Any`](Any).
    #[must_use]
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns the value as [`Box<dyn Any>`].
    #[must_use]
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Returns the value as [`Rc<dyn Any>`].
    #[must_use]
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
}

/// Upcasts the trait object to [`dyn Any + Send + Sync`](Any).
///
/// This trait is automatically implemented for any `&'static` type that implemenets the
/// [`Any`], [`Send`], [`Sync`] traits.
pub trait AsAnySync: AsAny + Send + Sync {
    /// Returns the value as [`&(dyn Any + Send + Sync)`](Any).
    #[must_use]
    fn as_any_sync(&self) -> &(dyn Any + Send + Sync);

    /// Returns the value as [`&mut (dyn Any + Send + Sync)`](Any).
    #[must_use]
    fn as_any_sync_mut(&mut self) -> &mut (dyn Any + Send + Sync);

    /// Returns the value as [`Box<dyn Any + Send + Sync>`].
    #[must_use]
    fn into_any_sync(self: Box<Self>) -> Box<dyn Any + Send + Sync>;

    /// Returns the value as [`Arc<dyn Any + Send + Sync>`].
    #[must_use]
    fn into_any_sync_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

/// An error type that is returened when failing to downcast trait objects.
#[derive(Clone, Copy, Debug)]
pub struct CastError {
    /// [`type name`](any::type_name) of the source type.
    pub src: &'static str,
    /// [`type name`](any::type_name) of the destination type.
    pub dst: &'static str,
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { src, dst } = *self;
        write!(f, "try to downcast `{src}` to `{dst}`")
    }
}

impl Error for CastError {}

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                      TRAIT IMPLEMENTATION                                      //
////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Any> AsAny for T {
    #[inline]
    fn type_name(&self) -> &'static str {
        any::type_name::<T>()
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    #[inline]
    fn into_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

impl<T: Any + Send + Sync> AsAnySync for T {
    #[inline]
    fn as_any_sync(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    #[inline]
    fn as_any_sync_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }

    #[inline]
    fn into_any_sync(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }

    #[inline]
    fn into_any_sync_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}
