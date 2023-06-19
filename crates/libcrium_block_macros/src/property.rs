use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Brace,
    Attribute, Error, Expr, Path, PathArguments, PathSegment, Result, Token, Type, Visibility,
};

pub struct MacroAttrs {
    crate_path: Option<Path>,
}

pub struct MacroInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    struct_token: Token![struct],
    ident: Ident,
    brace_token: Brace,
    fields: Punctuated<MacroField, Token![,]>,
}

pub struct MacroField {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    colon_token: Token![:],
    ty: Type,
    key: Expr,
}

impl Parse for MacroAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut crate_path = None;

        while !input.is_empty() {
            let _crate_token: Token![crate] = input.parse()?;
            let _equal_token: Token![=] = input.parse()?;
            crate_path = Some(input.parse()?);
        }

        Ok(Self { crate_path })
    }
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            brace_token: syn::braced!(content in input),
            fields: content.parse_terminated(MacroField::parse, Token![,])?,
        })
    }
}

impl Parse for MacroField {
    fn parse(input: ParseStream) -> Result<Self> {
        #[inline(always)]
        fn findattr(attr: &Attribute) -> bool {
            attr.path().is_ident("property")
        }

        let mut attrs = input.call(Attribute::parse_outer)?;
        let key = match attrs.iter().position(findattr) {
            Some(index) => attrs.swap_remove(index),
            None => return Err(Error::new(input.span(), "expects #[property = ...]")),
        };

        Ok(Self {
            attrs,
            vis: input.parse()?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            key: key.meta.require_name_value()?.value.clone(),
        })
    }
}

impl ToTokens for MacroInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in self.attrs.iter() {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);
        self.struct_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.brace_token
            .surround(tokens, |tokens| self.fields.to_tokens(tokens));
    }
}

impl ToTokens for MacroField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in self.attrs.iter() {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

pub(crate) fn expand_property(attrs: MacroAttrs, input: MacroInput) -> Result<TokenStream> {
    let crate_path = attrs.crate_path.unwrap_or_else(|| Path {
        leading_colon: None,
        segments: Punctuated::from_iter(std::iter::once(PathSegment {
            ident: Ident::new("libcrium_block", Span::call_site()),
            arguments: PathArguments::None,
        })),
    });

    let struct_name = &input.ident;
    let fields_count = input.fields.len();
    let mut fields_ident = Vec::with_capacity(fields_count);
    let mut fields_types = Vec::with_capacity(fields_count);
    let mut fields_attrs = Vec::with_capacity(fields_count);
    let mut fields_index = Vec::with_capacity(fields_count);

    for field in input.fields.iter() {
        // attr.meta.require_list()?.parse_nested_meta(|meta| {
        //     fields_attrs.push(meta.path);
        //     Ok(())
        // })?;

        // attr.meta.require_name_value()
        let ident = &field.ident;

        fields_index.push(Ident::new(&format!("__i{ident}"), ident.span()));
        fields_ident.push(ident);
        fields_types.push(&field.ty);
        fields_attrs.push(&field.key);
    }

    Ok(quote::quote! {
        #input

        impl #crate_path::property::Properties for #struct_name {
            fn definition() -> &'static #crate_path::property::StateDefinition<Self> {
                static DEFINITION: std::sync::OnceLock<#crate_path::property::StateDefinition<#struct_name>> =
                    std::sync::OnceLock::new();

                DEFINITION.get_or_init(|| {
                    #( let #fields_ident: &[#fields_types] = #crate_path::property::Property::range(& #fields_attrs); )*

                    #crate_path::property::StateDefinition::__new(
                        std::convert::From::from([ #( &#fields_attrs as _, )* ]),
                        std::iter::FromIterator::from_iter(
                            #crate_path::property::__StatePermutation::new([ #( #fields_ident.len(), )* ]).map(
                                |[ #( #fields_index, )* ]| Self {
                                    #( #fields_ident: #fields_ident[#fields_index], )*
                                },
                            ),
                        ),
                    )
                })
            }
        }

        impl #crate_path::property::__SpecIndex for #struct_name {
            fn spec_index(&self, index: &dyn #crate_path::property::ReflectProperty) -> std::option::Option<&dyn #crate_path::property::ReflectValue> {
                #( if #fields_attrs.eq(index) { return std::option::Option::Some(&self.#fields_ident); } )*
                std::option::Option::None
            }

            fn spec_index_mut(&mut self, index: &dyn #crate_path::property::ReflectProperty) -> std::option::Option<&mut dyn #crate_path::property::ReflectValue> {
                #( if #fields_attrs.eq(index) { return std::option::Option::Some(&mut self.#fields_ident); } )*
                std::option::Option::None
            }
        }
    })
}
