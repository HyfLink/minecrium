use std::any::{self, Any};
use std::rc::Rc;
use std::sync::Arc;

// re-exports
pub use minecrium_common_macros::{downcast, downcast_sync};

/// Upcasts the trait object to [`dyn Any`](Any).
///
/// This trait is automatically implemented for types that implemenet the [`Any`] trait.
#[downcast(crate = crate)]
pub trait AsAny: Any {
    /// Returns the [`type name`](any::type_name) as a string slice.
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

/// Upcasts the trait object to [`dyn Any + Send + Sync`](Any).
///
/// This trait is automatically implemented for types that implemenet the [`Any`], [`Send`],
/// [`Sync`] traits.
#[downcast_sync(crate = crate)]
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

#[cfg(test)]
mod tests {
    use crate::dynamic::*;

    #[test]
    fn test_downcast_sync() {
        let val: &dyn AsAnySync = &32_i32;
        assert_eq!(val.downcast_ref::<i32>(), Some(&32_i32));
        assert_eq!(val.downcast_ref::<char>(), None);

        let val: &mut dyn AsAnySync = &mut 32_i32;
        *val.downcast_mut().unwrap() = 10_i32;
        assert_eq!(val.downcast_ref::<i32>(), Some(&10_i32));

        let val: Box<dyn AsAnySync> = Box::new(32_i32);
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 32_i32);
        assert_eq!(*val.downcast::<i32>().ok().unwrap(), 32_i32);

        let val: Rc<dyn AsAnySync> = Rc::new(32_i32);
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 32_i32);
        assert_eq!(*val.downcast_rc::<i32>().ok().unwrap(), 32_i32);

        let val: Arc<dyn AsAnySync> = Arc::new(64_i32);
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 64_i32);
        assert_eq!(*val.downcast_arc::<i32>().ok().unwrap(), 64_i32);
    }
}
