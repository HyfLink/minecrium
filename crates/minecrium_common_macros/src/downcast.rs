use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Error, ItemTrait, Path, Result, Token};

pub struct AttributeArgs {
    crate_path: Option<Path>,
}

impl AttributeArgs {
    fn take_crate_path(&mut self) -> Path {
        match self.crate_path.take() {
            Some(crate_path) => crate_path,
            None => minecrium_macro_utils::get_crate_path("minecrium_common"),
        }
    }
}

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        const MESSAGE: &str = "unexpected meta, expects `crate = ...`";

        if input.is_empty() {
            return Ok(Self { crate_path: None });
        }

        input.step(|cur| match cur.ident() {
            Some((ident, cur)) if ident == "crate" => Ok((ident, cur)),
            _ => Err(cur.error(MESSAGE)),
        })?;

        let _: Token![=] = input.parse()?;
        let path = input.parse()?;

        if !input.is_empty() {
            return Err(Error::new(input.span(), MESSAGE));
        }

        Ok(Self {
            crate_path: Some(path),
        })
    }
}

pub fn proc_macro_downcast(mut attrs: AttributeArgs, item: ItemTrait) -> TokenStream {
    let crate_path = attrs.take_crate_path();
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let trait_name = &item.ident;
    let trait_name = quote::quote!(#trait_name #type_generics);
    let impl_downcast = impl_downcast(&crate_path, &trait_name);

    quote::quote!(#item impl #impl_generics dyn #trait_name #where_clause { #impl_downcast })
}

pub fn proc_macro_downcast_sync(mut attrs: AttributeArgs, item: ItemTrait) -> TokenStream {
    let crate_path = attrs.take_crate_path();
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let trait_name = &item.ident;
    let trait_name = quote::quote!(#trait_name #type_generics);
    let impl_downcast = impl_downcast(&crate_path, &trait_name);
    let impl_downcast_sync = impl_downcast_sync(&crate_path, &trait_name);

    quote::quote!(#item impl #impl_generics dyn #trait_name #where_clause { #impl_downcast #impl_downcast_sync })
}

fn impl_downcast(crate_path: &Path, trait_name: &TokenStream) -> TokenStream {
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
            #crate_path::dynamic::AsAny::as_any(self).downcast_ref()
        }
        /// Returns the downcast value as `&mut T`.
        ///
        /// Returns `None` if `self.is::<T>()` evaluates to `false`.
        #[inline]
        pub fn downcast_mut<T: #trait_name>(&mut self) -> std::option::Option<&mut T> {
            #crate_path::dynamic::AsAny::as_any_mut(self).downcast_mut()
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
                // SAFETY: `inner` is just returned from `Box::into_raw`.
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
                // SAFETY: `inner` is just returned from `Rc::into_raw`.
                Ok(unsafe { std::rc::Rc::from_raw(inner) })
            } else {
                Err(self)
            }
        }
    }
}

fn impl_downcast_sync(_: &Path, trait_name: &TokenStream) -> TokenStream {
    quote::quote! {
        /// Returns the downcast value as [`Arc<T>`](std::sync::Arc).
        ///
        /// Returns the trait object if `self.is::<T>()` evaluates to `false`.
        #[inline]
        #[rustfmt::skip]
        pub fn downcast_arc<T: #trait_name>(self: std::sync::Arc<Self>) -> std::result::Result<std::sync::Arc<T>, std::sync::Arc<dyn #trait_name>> {
            if self.is::<T>() {
                let inner = std::sync::Arc::into_raw(self) as *const T;
                // SAFETY: `inner` is just returned from `Arc::into_raw`.
                Ok(unsafe { std::sync::Arc::from_raw(inner) })
            } else {
                Err(self)
            }
        }
    }
}
