//! Code generation for the derive macros.

use crate::attrs::{Container, Field, Variant};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, GenericParam, Generics, Ident, Index, LitInt,
    Result, Type, WherePredicate, parse_quote,
};

pub fn encode(input: DeriveInput) -> Result<TokenStream> {
    let container = Container::parse(&input.attrs)?;
    let krate = &container.krate;
    let name = &input.ident;
    let mut generics = input.generics.clone();
    apply_encode_bounds(&mut generics, &container);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data) => encode_struct(data, krate)?,
        Data::Enum(data) => encode_enum(data, krate)?,
        Data::Union(_) => {
            return Err(syn::Error::new(
                Span::call_site(),
                "unions are not supported",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics #krate::Encode for #name #ty_generics #where_clause {
            fn encode<__E: #krate::enc::Encoder>(
                &self,
                encoder: &mut __E,
            ) -> core::result::Result<(), #krate::error::EncodeError> {
                #body
            }
        }
    })
}

pub fn decode(input: DeriveInput) -> Result<TokenStream> {
    let container = Container::parse(&input.attrs)?;
    let decode = expand_decode(&input, &container)?;
    let borrow = expand_borrow_decode(&input, &container)?;
    Ok(quote! {
        #decode
        #borrow
    })
}

pub fn borrow_decode(input: DeriveInput) -> Result<TokenStream> {
    let container = Container::parse(&input.attrs)?;
    expand_borrow_decode(&input, &container)
}

fn encode_struct(data: &DataStruct, krate: &syn::Path) -> Result<TokenStream> {
    let mut statements = TokenStream::new();
    for (index, field) in data.fields.iter().enumerate() {
        let attributes = Field::parse(&field.attrs)?;
        if attributes.skip {
            continue;
        }
        let access = field_access(&field.ident, index);
        let statement = encode_field(&attributes, krate, quote!(&self.#access))?;
        statements.extend(statement);
    }
    Ok(quote! {
        #statements
        core::result::Result::Ok(())
    })
}

fn encode_enum(data: &DataEnum, krate: &syn::Path) -> Result<TokenStream> {
    if data.variants.is_empty() {
        return Ok(quote! {
            match self {
                _ => core::unreachable!(),
            }
        });
    }

    let mut arms = TokenStream::new();
    for (index, variant) in data.variants.iter().enumerate() {
        let variant_name = &variant.ident;
        let variant_attributes = Variant::parse(&variant.attrs)?;
        let index = LitInt::new(&format!("{index}u32"), Span::call_site());

        let (pattern, bindings) = variant_pattern(variant_name, &variant.fields)?;
        let mut statements = quote! {
            <u32 as #krate::Encode>::encode(&#index, encoder)?;
        };

        if let Some(with) = &variant_attributes.with {
            let bindings = bindings.iter();
            statements.extend(quote! {
                #with::encode(#(#bindings,)* encoder)?;
            });
        } else {
            let mut binding_index = 0;
            for field in variant.fields.iter() {
                let attributes = Field::parse(&field.attrs)?;
                if attributes.skip {
                    continue;
                }
                let binding = &bindings[binding_index];
                binding_index += 1;
                let statement = encode_field(&attributes, krate, quote!(#binding))?;
                statements.extend(statement);
            }
        }

        arms.extend(quote! {
            #pattern => {
                #statements
                core::result::Result::Ok(())
            }
        });
    }

    Ok(quote! {
        match self {
            #arms
        }
    })
}

fn encode_field(attributes: &Field, krate: &syn::Path, value: TokenStream) -> Result<TokenStream> {
    if let Some(with) = &attributes.with {
        Ok(quote! {
            #with::encode(#value, encoder)?;
        })
    } else if attributes.with_serde {
        Ok(quote! {
            #krate::Encode::encode(&#krate::serde::Compat(#value), encoder)?;
        })
    } else {
        Ok(quote! {
            #krate::Encode::encode(#value, encoder)?;
        })
    }
}

fn expand_decode(input: &DeriveInput, container: &Container) -> Result<TokenStream> {
    let krate = &container.krate;
    let name = &input.ident;
    let mut generics = input.generics.clone();
    let (context, added) = context_type(&mut generics, container);
    apply_decode_bounds(&mut generics, container, &context, added)?;
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty_generics = quote!(#ty_generics);

    let body = match &input.data {
        Data::Struct(data) => decode_struct(data, krate, &context)?,
        Data::Enum(data) => decode_enum(data, krate, &context, name)?,
        Data::Union(_) => {
            return Err(syn::Error::new(
                Span::call_site(),
                "unions are not supported",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics #krate::Decode<#context> for #name #ty_generics #where_clause {
            fn decode<__D: #krate::de::Decoder<Context = #context>>(
                decoder: &mut __D,
            ) -> core::result::Result<Self, #krate::error::DecodeError> {
                #body
            }
        }
    })
}

fn expand_borrow_decode(input: &DeriveInput, container: &Container) -> Result<TokenStream> {
    let krate = &container.krate;
    let name = &input.ident;
    let mut generics = input.generics.clone();
    let (context, added) = context_type(&mut generics, container);
    let lifetimes = original_lifetimes(&input.generics);
    generics
        .params
        .insert(0, GenericParam::Lifetime(parse_quote!('__de)));
    apply_borrow_bounds(&mut generics, container, &context, added, &lifetimes)?;
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty_generics = quote!(#ty_generics);

    let body = borrow_body(input, krate, &context, name)?;

    Ok(quote! {
        impl #impl_generics #krate::BorrowDecode<'__de, #context> for #name #ty_generics #where_clause {
            fn borrow_decode<__D: #krate::de::BorrowDecoder<'__de, Context = #context>>(
                decoder: &mut __D,
            ) -> core::result::Result<Self, #krate::error::DecodeError> {
                #body
            }
        }
    })
}

fn borrow_body(
    input: &DeriveInput,
    krate: &syn::Path,
    context: &Type,
    name: &Ident,
) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(data) => decode_struct_borrow(data, krate, context),
        Data::Enum(data) => decode_enum_borrow(data, krate, context, name),
        Data::Union(_) => unreachable!(),
    }
}

fn decode_struct(data: &DataStruct, krate: &syn::Path, context: &Type) -> Result<TokenStream> {
    let mut fields = TokenStream::new();
    for (index, field) in data.fields.iter().enumerate() {
        let attributes = Field::parse(&field.attrs)?;
        let value = decode_field(&attributes, krate, context, false)?;
        match &field.ident {
            Some(ident) => fields.extend(quote!(#ident: #value,)),
            None => {
                let _ = index;
                fields.extend(quote!(#value,));
            }
        }
    }
    Ok(match &data.fields {
        Fields::Named(_) => quote!(core::result::Result::Ok(Self { #fields })),
        Fields::Unnamed(_) => quote!(core::result::Result::Ok(Self(#fields))),
        Fields::Unit => quote!(core::result::Result::Ok(Self)),
    })
}

fn decode_struct_borrow(
    data: &DataStruct,
    krate: &syn::Path,
    context: &Type,
) -> Result<TokenStream> {
    let mut fields = TokenStream::new();
    for field in data.fields.iter() {
        let attributes = Field::parse(&field.attrs)?;
        let value = decode_field(&attributes, krate, context, true)?;
        match &field.ident {
            Some(ident) => fields.extend(quote!(#ident: #value,)),
            None => fields.extend(quote!(#value,)),
        }
    }
    Ok(match &data.fields {
        Fields::Named(_) => quote!(core::result::Result::Ok(Self { #fields })),
        Fields::Unnamed(_) => quote!(core::result::Result::Ok(Self(#fields))),
        Fields::Unit => quote!(core::result::Result::Ok(Self)),
    })
}

fn decode_field(
    attributes: &Field,
    krate: &syn::Path,
    context: &Type,
    borrow: bool,
) -> Result<TokenStream> {
    if attributes.skip {
        if let Some(default) = &attributes.default {
            return Ok(quote!((#default)()));
        }
        return Ok(quote!(core::default::Default::default()));
    }

    if let Some(with) = &attributes.with {
        return if borrow {
            Ok(quote!(#with::borrow_decode(decoder)?))
        } else {
            Ok(quote!(#with::decode(decoder)?))
        };
    }

    if attributes.with_serde {
        return if borrow {
            Ok(quote!(
                <#krate::serde::BorrowCompat<_> as #krate::BorrowDecode<'_, #context>>::borrow_decode(decoder)?.0
            ))
        } else {
            Ok(quote!(
                <#krate::serde::Compat<_> as #krate::Decode<#context>>::decode(decoder)?.0
            ))
        };
    }

    Ok(if borrow {
        quote!(#krate::BorrowDecode::<'_, #context>::borrow_decode(decoder)?)
    } else {
        quote!(#krate::Decode::<#context>::decode(decoder)?)
    })
}

fn decode_enum(
    data: &DataEnum,
    krate: &syn::Path,
    context: &Type,
    name: &Ident,
) -> Result<TokenStream> {
    if data.variants.is_empty() {
        return Ok(quote! {
            core::result::Result::Err(#krate::error::DecodeError::EmptyEnum {
                type_name: core::any::type_name::<Self>(),
            })
        });
    }

    let mut arms = TokenStream::new();
    for (index, variant) in data.variants.iter().enumerate() {
        let attributes = Variant::parse(&variant.attrs)?;
        let index = LitInt::new(&format!("{index}u32"), Span::call_site());
        let value = if let Some(with) = &attributes.with {
            quote!(#with::decode(decoder))
        } else {
            let mut fields = TokenStream::new();
            for (field_index, field) in variant.fields.iter().enumerate() {
                let attributes = Field::parse(&field.attrs)?;
                let value = decode_field(&attributes, krate, context, false)?;
                match &field.ident {
                    Some(ident) => fields.extend(quote!(#ident: #value,)),
                    None => {
                        let _ = field_index;
                        fields.extend(quote!(#value,));
                    }
                }
            }
            let variant_name = &variant.ident;
            match &variant.fields {
                Fields::Named(_) => {
                    quote!(core::result::Result::Ok(Self::#variant_name { #fields }))
                }
                Fields::Unnamed(_) => {
                    quote!(core::result::Result::Ok(Self::#variant_name(#fields)))
                }
                Fields::Unit => quote!(core::result::Result::Ok(Self::#variant_name)),
            }
        };
        arms.extend(quote!(#index => #value,));
    }

    let invalid = invalid_variant(data, krate, name);
    Ok(quote! {
        let variant = <u32 as #krate::Decode<#context>>::decode(decoder)?;
        match variant {
            #arms
            #invalid
        }
    })
}

fn decode_enum_borrow(
    data: &DataEnum,
    krate: &syn::Path,
    context: &Type,
    name: &Ident,
) -> Result<TokenStream> {
    if data.variants.is_empty() {
        return Ok(quote! {
            core::result::Result::Err(#krate::error::DecodeError::EmptyEnum {
                type_name: core::any::type_name::<Self>(),
            })
        });
    }

    let mut arms = TokenStream::new();
    for (index, variant) in data.variants.iter().enumerate() {
        let attributes = Variant::parse(&variant.attrs)?;
        let index = LitInt::new(&format!("{index}u32"), Span::call_site());
        let value = if let Some(with) = &attributes.with {
            quote!(#with::borrow_decode(decoder))
        } else {
            let mut fields = TokenStream::new();
            for field in variant.fields.iter() {
                let attributes = Field::parse(&field.attrs)?;
                let value = decode_field(&attributes, krate, context, true)?;
                match &field.ident {
                    Some(ident) => fields.extend(quote!(#ident: #value,)),
                    None => fields.extend(quote!(#value,)),
                }
            }
            let variant_name = &variant.ident;
            match &variant.fields {
                Fields::Named(_) => {
                    quote!(core::result::Result::Ok(Self::#variant_name { #fields }))
                }
                Fields::Unnamed(_) => {
                    quote!(core::result::Result::Ok(Self::#variant_name(#fields)))
                }
                Fields::Unit => quote!(core::result::Result::Ok(Self::#variant_name)),
            }
        };
        arms.extend(quote!(#index => #value,));
    }

    let invalid = invalid_variant(data, krate, name);
    Ok(quote! {
        let variant = <u32 as #krate::Decode<#context>>::decode(decoder)?;
        match variant {
            #arms
            #invalid
        }
    })
}

fn invalid_variant(data: &DataEnum, krate: &syn::Path, name: &Ident) -> TokenStream {
    let allowed = if data
        .variants
        .iter()
        .any(|variant| variant.discriminant.is_some())
    {
        let values = (0..data.variants.len())
            .map(|index| LitInt::new(&format!("{index}u32"), Span::call_site()));
        quote!(&#krate::error::AllowedEnumVariants::Allowed(&[#(#values),*]))
    } else {
        let max = LitInt::new(
            &format!("{}u32", data.variants.len() - 1),
            Span::call_site(),
        );
        quote!(&#krate::error::AllowedEnumVariants::Range { min: 0, max: #max })
    };

    quote! {
        _ => core::result::Result::Err(#krate::error::DecodeError::UnexpectedVariant {
            found: variant,
            type_name: stringify!(#name),
            allowed: #allowed,
        }),
    }
}

fn variant_pattern(name: &Ident, fields: &Fields) -> Result<(TokenStream, Vec<Ident>)> {
    let mut bindings = Vec::new();
    let pattern = match fields {
        Fields::Named(fields) => {
            let mut patterns = TokenStream::new();
            for field in &fields.named {
                let attributes = Field::parse(&field.attrs)?;
                let ident = field.ident.as_ref().expect("named field");
                if attributes.skip {
                    patterns.extend(quote!(#ident: _,));
                } else {
                    patterns.extend(quote!(#ident,));
                    bindings.push(ident.clone());
                }
            }
            quote!(Self::#name { #patterns })
        }
        Fields::Unnamed(fields) => {
            let mut patterns = TokenStream::new();
            for (index, field) in fields.unnamed.iter().enumerate() {
                let attributes = Field::parse(&field.attrs)?;
                if attributes.skip {
                    patterns.extend(quote!(_,));
                } else {
                    let binding = format_ident!("field_{index}");
                    patterns.extend(quote!(#binding,));
                    bindings.push(binding);
                }
            }
            quote!(Self::#name(#patterns))
        }
        Fields::Unit => quote!(Self::#name),
    };
    Ok((pattern, bindings))
}

fn field_access(ident: &Option<Ident>, index: usize) -> TokenStream {
    match ident {
        Some(ident) => quote!(#ident),
        None => {
            let index = Index::from(index);
            quote!(#index)
        }
    }
}

fn context_type(generics: &mut Generics, container: &Container) -> (Type, bool) {
    match &container.decode_context {
        Some(context) => (context.clone(), false),
        None => {
            generics
                .params
                .push(GenericParam::Type(parse_quote!(__Context)));
            (parse_quote!(__Context), true)
        }
    }
}

fn apply_encode_bounds(generics: &mut Generics, container: &Container) {
    if let Some(bounds) = container
        .encode_bounds
        .as_ref()
        .or(container.bounds.as_ref())
    {
        replace_where(generics, bounds);
    } else {
        let krate = &container.krate;
        let predicates: Vec<WherePredicate> = type_params(generics)
            .map(|ident| parse_quote!(#ident: #krate::Encode))
            .collect();
        extend_where(generics, predicates.into_iter());
    }
}

fn apply_decode_bounds(
    generics: &mut Generics,
    container: &Container,
    context: &Type,
    _added: bool,
) -> Result<()> {
    if let Some(bounds) = container
        .decode_bounds
        .as_ref()
        .or(container.bounds.as_ref())
    {
        replace_where(generics, bounds);
    } else {
        let krate = &container.krate;
        let predicates: Vec<WherePredicate> = type_params(generics)
            .map(|ident| parse_quote!(#ident: #krate::Decode<#context>))
            .collect();
        extend_where(generics, predicates.into_iter());
    }
    Ok(())
}

fn apply_borrow_bounds(
    generics: &mut Generics,
    container: &Container,
    context: &Type,
    _added: bool,
    lifetimes: &[syn::Lifetime],
) -> Result<()> {
    if let Some(bounds) = container
        .borrow_decode_bounds
        .as_ref()
        .or(container.bounds.as_ref())
    {
        replace_where(generics, bounds);
    } else {
        let krate = &container.krate;
        let predicates: Vec<WherePredicate> = type_params(generics)
            .map(|ident| parse_quote!(#ident: #krate::BorrowDecode<'__de, #context> + '__de))
            .collect();
        extend_where(generics, predicates.into_iter());
        let lifetime_bounds: Vec<WherePredicate> = lifetimes
            .iter()
            .map(|lifetime| parse_quote!('__de: #lifetime))
            .collect();
        extend_where(generics, lifetime_bounds.into_iter());
    }
    Ok(())
}

fn type_params(generics: &Generics) -> impl Iterator<Item = &Ident> {
    generics.params.iter().filter_map(|param| match param {
        GenericParam::Type(param) if param.ident != "__Context" => Some(&param.ident),
        _ => None,
    })
}

fn original_lifetimes(generics: &Generics) -> Vec<syn::Lifetime> {
    generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Lifetime(param) => Some(param.lifetime.clone()),
            _ => None,
        })
        .collect()
}

fn replace_where(
    generics: &mut Generics,
    predicates: &syn::punctuated::Punctuated<WherePredicate, syn::Token![,]>,
) {
    let where_clause = generics.make_where_clause();
    where_clause.predicates = predicates.clone();
}

fn extend_where(generics: &mut Generics, predicates: impl Iterator<Item = WherePredicate>) {
    let where_clause = generics.make_where_clause();
    for predicate in predicates {
        where_clause.predicates.push(predicate);
    }
}
