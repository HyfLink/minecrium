use proc_macro2::{Ident, Span, TokenStream};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Error, Generics, LitStr, Path, PathArguments, PathSegment, Result, Token, Visibility,
};

pub struct StrEnumAttrs {
    pub crate_path: Option<Path>,
    pub error: Option<Ident>,
}

pub struct StrEnumInput {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub variants: Punctuated<StrEnumVariant, Token![,]>,
}

pub struct StrEnumVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub discriminant: LitStr,
}

impl Parse for StrEnumAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut crate_path: Option<Path> = None;
        let mut error: Option<Ident> = None;

        while !input.is_empty() {
            if input.peek(Token![crate]) {
                input.parse::<Token![crate]>()?;
                input.parse::<Token![=]>()?;
                crate_path = Some(input.parse()?);
            } else {
                let ident: Ident = input.parse()?;

                if ident == "error" {
                    input.parse::<Token![=]>()?;
                    error = Some(input.parse()?);
                } else {
                    let message = "expects meta `crate` or `error`";
                    return Err(Error::new(input.span(), message));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { crate_path, error })
    }
}

impl Parse for StrEnumInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            ident: {
                input.parse::<Token![enum]>()?;
                input.parse()?
            },
            generics: {
                let mut generics: Generics = input.parse()?;
                generics.where_clause = input.parse()?;
                generics
            },
            variants: {
                let content;
                syn::braced!(content in input);
                content.parse_terminated(StrEnumVariant::parse, Token![,])?
            },
        })
    }
}

impl Parse for StrEnumVariant {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ident: input.parse()?,
            discriminant: {
                input.parse::<Token![=]>()?;
                input.parse()?
            },
        })
    }
}

pub fn expand_strenum(attrs: StrEnumAttrs, input: StrEnumInput) -> TokenStream {
    let crate_path = attrs.crate_path.unwrap_or_else(|| Path {
        leading_colon: None,
        segments: Punctuated::from_iter(std::iter::once(PathSegment {
            ident: Ident::new("libcrium_core", Span::call_site()),
            arguments: PathArguments::None,
        })),
    });

    let enum_attrs = input.attrs;
    let enum_vis = input.vis;
    let enum_name = input.ident;
    let enum_generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = enum_generics.split_for_impl();

    let enum_error = attrs.error.clone().unwrap_or_else(|| -> Ident {
        Ident::new(&format!("{enum_name}FromStrError"), enum_name.span())
    });

    let impl_enum_error: Option<TokenStream> = attrs.error.is_none().then(|| quote::quote! {
        /// An error that is [`FromStr::Err`](std::str::FromStr::Err).
        #[doc(hidden)]
        #[derive(Clone, Copy, Debug, Default)]
        pub struct #enum_error;

        impl std::fmt::Display for #enum_error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::write!(f, "cannot parse {}", std::stringify!(#enum_name))?;
                if f.alternate() {
                    std::write!(f, ", expects one of {:?}", <#enum_name as #crate_path::strenum::StrEnum>::VALUES)?;
                }
                Ok(())
            }
        }

        impl std::error::Error for #enum_error {}
    });

    let mut variants_attrs: Vec<Vec<Attribute>> = Vec::with_capacity(input.variants.len());
    let mut variants_ident: Vec<Ident> = Vec::with_capacity(input.variants.len());
    let mut variants_names: Vec<LitStr> = Vec::with_capacity(input.variants.len());

    for variant in input.variants.into_iter() {
        variants_attrs.push(variant.attrs);
        variants_ident.push(variant.ident);
        variants_names.push(variant.discriminant);
    }

    quote::quote! {
        #( #enum_attrs )*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        #enum_vis enum #enum_name #enum_generics {
            #( #( #variants_attrs )* #variants_ident, )*
        }

        impl #impl_generics #crate_path::strenum::DynEnum for #enum_name #type_generics #where_clause {
            fn as_str(&self) -> &'static str {
                match self { #( Self::#variants_ident => #variants_names, )* }
            }
        }

        impl #impl_generics #crate_path::strenum::StrEnum for #enum_name #type_generics #where_clause {
            type FromStrError = <Self as std::str::FromStr>::Err;
            const VALUES: &'static [Self] = &[ #( Self::#variants_ident, )* ];
        }

        impl #impl_generics std::convert::AsRef<str> for #enum_name #type_generics #where_clause {
            #[inline]
            fn as_ref(&self) -> &str {
                <Self as #crate_path::strenum::DynEnum>::as_str(self)
            }
        }

        impl #impl_generics std::fmt::Debug for #enum_name #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(<Self as #crate_path::strenum::DynEnum>::as_str(self))
            }
        }

        impl #impl_generics std::fmt::Display for #enum_name #type_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(<Self as #crate_path::strenum::DynEnum>::as_str(self))
            }
        }

        impl #impl_generics std::str::FromStr for #enum_name #type_generics #where_clause {
            type Err = #enum_error;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                match s {
                    #( #variants_names => std::result::Result::Ok(Self::#variants_ident), )*
                    _ => std::result::Result::Err(#enum_error),
                }
            }
        }

        #impl_enum_error
    }
}
