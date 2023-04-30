use proc_macro2::TokenStream;
use syn::ItemTrait;

pub(crate) fn proc_macro_downcast(item: ItemTrait) -> TokenStream {
    let crate_path = minecrium_macro_utils::get_minecrium_path("minecrium_core");
    let trait_name = &item.ident;

    quote::quote! {
        #item

        impl dyn #trait_name {
            /// Returns `true` if the inner type is the same as `T`.
            #[inline(always)]
            pub fn is<T: #trait_name>(&self) -> bool {
                #crate_path::dynamic::AsAny::as_any(self).is::<T>()
            }

            /// Returns the downcast value as `&T`.
            ///
            /// Returns `None` if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_ref<T: #trait_name>(&self) -> std::option::Option<&T> {
                #crate_path::dynamic::AsAny::as_any(self).downcast_ref()
            }

            /// Returns the downcast value as `&mut T`.
            ///
            /// # Errors
            ///
            /// Returns `None` if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_mut<T: #trait_name>(&mut self) -> std::option::Option<&mut T> {
                #crate_path::dynamic::AsAny::as_any_mut(self).downcast_mut()
            }

            /// Returns the downcast value as [`Rc<T>`](std::rc::Rc).
            ///
            /// Returns the trait object as an error if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_rc<T: #trait_name>(
                self: std::rc::Rc<Self>,
            ) -> std::result::Result<std::rc::Rc<T>, std::rc::Rc<dyn std::any::Any>> {
                #crate_path::dynamic::AsAny::into_any_rc(self).downcast()
            }

            /// Returns the downcast value as [`Box<T>`](std::boxed::Box).
            ///
            /// Returns the trait object as an error if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_boxed<T: #trait_name>(
                self: std::boxed::Box<Self>,
            ) -> std::result::Result<std::boxed::Box<T>, std::boxed::Box<dyn std::any::Any>> {
                #crate_path::dynamic::AsAny::into_any(self).downcast()
            }
        }
    }
}

pub(crate) fn proc_macro_downcast_sync(item: ItemTrait) -> TokenStream {
    let crate_path = minecrium_macro_utils::get_minecrium_path("minecrium_core");
    let trait_name = &item.ident;

    quote::quote! {
        #item

        impl dyn #trait_name {
            /// Returns `true` if the inner type is the same as `T`.
            #[inline(always)]
            pub fn is<T: #trait_name>(&self) -> bool {
                #crate_path::dynamic::AsAnySync::as_any_sync(self).is::<T>()
            }

            /// Returns the downcast value as `&T`.
            ///
            /// Returns `None` if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_ref<T: #trait_name>(&self) -> std::option::Option<&T> {
                #crate_path::dynamic::AsAnySync::as_any_sync(self).downcast_ref()
            }

            /// Returns the downcast value as `&mut T`.
            ///
            /// # Errors
            ///
            /// Returns `None` if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_mut<T: #trait_name>(&mut self) -> std::option::Option<&mut T> {
                #crate_path::dynamic::AsAnySync::as_any_sync_mut(self).downcast_mut()
            }

            /// Returns the downcast value as [`Rc<T>`](std::rc::Rc).
            ///
            /// Returns the trait object as an error if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_rc<T: #trait_name>(
                self: std::rc::Rc<Self>,
            ) -> std::result::Result<std::rc::Rc<T>, std::rc::Rc<dyn std::any::Any>> {
                #crate_path::dynamic::AsAny::into_any_rc(self).downcast()
            }

            /// Returns the downcast value as [`Arc<T>`](std::sync::Arc).
            ///
            /// Returns the trait object as an error if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_arc<T: #trait_name>(
                self: std::sync::Arc<Self>,
            ) -> std::result::Result<std::sync::Arc<T>, std::sync::Arc<dyn std::any::Any + Send + Sync>> {
                #crate_path::dynamic::AsAnySync::into_any_sync_arc(self).downcast()
            }

            /// Returns the downcast value as [`Box<T>`](std::boxed::Box).
            ///
            /// Returns the trait object as an error if `self.is::<T>()` evaluates to `false`.
            #[inline(always)]
            pub fn downcast_boxed<T: #trait_name>(
                self: std::boxed::Box<Self>,
            ) -> std::result::Result<std::boxed::Box<T>, std::boxed::Box<dyn std::any::Any + Send + Sync>> {
                #crate_path::dynamic::AsAnySync::into_any_sync(self).downcast()
            }
        }
    }
}
