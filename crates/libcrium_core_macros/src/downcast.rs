use proc_macro2::{Ident, TokenStream};

pub fn expand_downcast(item: syn::ItemTrait) -> TokenStream {
    let trait_name = &item.ident;
    let impl_downcast = impl_downcast(trait_name);

    quote::quote!(#item impl dyn #trait_name { #impl_downcast })
}

pub fn expand_downcast_sync(item: syn::ItemTrait) -> TokenStream {
    let trait_name = &item.ident;
    let impl_downcast = impl_downcast(trait_name);
    let impl_downcast_sync = impl_downcast_sync(trait_name);

    quote::quote!(#item impl dyn #trait_name { #impl_downcast #impl_downcast_sync })
}

fn impl_downcast(trait_name: &Ident) -> TokenStream {
    quote::quote! {
        /// Returns `true` if the inner type is the same as `T`.
        #[inline]
        pub fn is<T: #trait_name>(&self) -> bool {
            std::any::Any::type_id(self) == std::any::TypeId::of::<T>()
        }
        /// Returns the downcast value as `&T`.
        ///
        /// Returns `None` if `self.is::<T>()` evaluates to `false`.
        #[inline]
        pub fn downcast_ref<T: #trait_name>(&self) -> std::option::Option<&T> {
            if self.is::<T>() {
                // SAFETY: the inner type of `self` is checked to be `T`.
                Some(unsafe { &*(self as *const dyn #trait_name as *const T) })
            } else {
                None
            }
        }
        /// Returns the downcast value as `&mut T`.
        ///
        /// Returns `None` if `self.is::<T>()` evaluates to `false`.
        #[inline]
        pub fn downcast_mut<T: #trait_name>(&mut self) -> std::option::Option<&mut T> {
            if self.is::<T>() {
                Some(unsafe { &mut *(self as *mut dyn #trait_name as *mut T) })
            } else {
                None
            }
        }
        /// Returns the downcast value as [`Box<T>`](std::boxed::Box).
        ///
        /// # Errors
        ///
        /// Returns the trait object if `self.is::<T>()` evaluates to `false`.
        #[inline]
        pub fn downcast<T: #trait_name>(self: std::boxed::Box<Self>) -> std::result::Result<std::boxed::Box<T>, std::boxed::Box<dyn #trait_name>> {
            if self.is::<T>() {
                let inner = std::boxed::Box::into_raw(self) as *mut T;
                // SAFETY: the inner type of `self` is checked to be `T`.
                Ok(unsafe { std::boxed::Box::from_raw(inner) })
            } else {
                Err(self)
            }
        }
        /// Returns the downcast value as [`Rc<T>`](std::rc::Rc).
        ///
        /// Returns the trait object if `self.is::<T>()` evaluates to `false`.
        #[inline]
        pub fn downcast_rc<T: #trait_name>(self: std::rc::Rc<Self>) -> std::result::Result<std::rc::Rc<T>, std::rc::Rc<dyn #trait_name>> {
            if self.is::<T>() {
                let inner = std::rc::Rc::into_raw(self) as *const T;
                // SAFETY: the inner type of `self` is checked to be `T`.
                Ok(unsafe { std::rc::Rc::from_raw(inner) })
            } else {
                Err(self)
            }
        }
    }
}

fn impl_downcast_sync(trait_name: &Ident) -> TokenStream {
    quote::quote! {
        /// Returns the downcast value as [`Arc<T>`](std::sync::Arc).
        ///
        /// Returns the trait object if `self.is::<T>()` evaluates to `false`.
        #[inline]
        #[rustfmt::skip]
        pub fn downcast_arc<T: #trait_name>(self: std::sync::Arc<Self>) -> std::result::Result<std::sync::Arc<T>, std::sync::Arc<dyn #trait_name>> {
            if self.is::<T>() {
                let inner = std::sync::Arc::into_raw(self) as *const T;
                // SAFETY: the inner type of `self` is checked to be `T`.
                Ok(unsafe { std::sync::Arc::from_raw(inner) })
            } else {
                Err(self)
            }
        }
    }
}
