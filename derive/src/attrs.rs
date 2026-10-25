//! Attribute parsing for the derive macros.

use syn::{
    Attribute, Error, LitStr, Path, Result, Token, Type, WherePredicate, meta::ParseNestedMeta,
    punctuated::Punctuated,
};

#[derive(Clone)]
pub struct Container {
    pub krate: Path,
    pub bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    pub encode_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    pub decode_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    pub borrow_decode_bounds: Option<Punctuated<WherePredicate, Token![,]>>,
    pub decode_context: Option<Type>,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            krate: syn::parse_quote!(::uklad),
            bounds: None,
            encode_bounds: None,
            decode_bounds: None,
            borrow_decode_bounds: None,
            decode_context: None,
        }
    }
}

impl Container {
    pub fn parse(attributes: &[Attribute]) -> Result<Self> {
        let mut container = Self::default();
        for attribute in attributes {
            if !attribute.path().is_ident("uklad") {
                continue;
            }
            attribute.parse_nested_meta(|meta| container.apply(&meta))?;
        }
        Ok(container)
    }

    fn apply(&mut self, meta: &ParseNestedMeta<'_>) -> Result<()> {
        if meta.path.is_ident("crate") {
            let literal = literal(meta)?;
            self.krate = literal.parse()?;
        } else if meta.path.is_ident("bounds") {
            self.bounds = Some(predicates(meta)?);
        } else if meta.path.is_ident("encode_bounds") {
            self.encode_bounds = Some(predicates(meta)?);
        } else if meta.path.is_ident("decode_bounds") {
            self.decode_bounds = Some(predicates(meta)?);
        } else if meta.path.is_ident("borrow_decode_bounds") {
            self.borrow_decode_bounds = Some(predicates(meta)?);
        } else if meta.path.is_ident("decode_context") {
            self.decode_context = Some(literal(meta)?.parse()?);
        } else {
            return Err(meta.error("unknown container attribute"));
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Field {
    pub skip: bool,
    pub with: Option<Path>,
    pub with_serde: bool,
    pub default: Option<Path>,
}

impl Field {
    pub fn parse(attributes: &[Attribute]) -> Result<Self> {
        let mut field = Self::default();
        for attribute in attributes {
            if !attribute.path().is_ident("uklad") {
                continue;
            }
            attribute.parse_nested_meta(|meta| field.apply(&meta))?;
        }
        if let Some(default) = field.default.as_ref().filter(|_| !field.skip) {
            return Err(Error::new_spanned(default, "`default` requires `skip`"));
        }
        Ok(field)
    }

    fn apply(&mut self, meta: &ParseNestedMeta<'_>) -> Result<()> {
        if meta.path.is_ident("skip") {
            if self.with.is_some() {
                return Err(meta.error("`skip` and `with` are mutually exclusive"));
            }
            if self.with_serde {
                return Err(meta.error("`skip` and `with_serde` are mutually exclusive"));
            }
            self.skip = true;
        } else if meta.path.is_ident("with_serde") {
            if self.skip {
                return Err(meta.error("`with_serde` and `skip` are mutually exclusive"));
            }
            if self.with.is_some() {
                return Err(meta.error("`with_serde` and `with` are mutually exclusive"));
            }
            self.with_serde = true;
        } else if meta.path.is_ident("with") {
            if self.skip {
                return Err(meta.error("`with` and `skip` are mutually exclusive"));
            }
            if self.with_serde {
                return Err(meta.error("`with` and `with_serde` are mutually exclusive"));
            }
            self.with = Some(literal(meta)?.parse()?);
        } else if meta.path.is_ident("default") {
            self.default = Some(literal(meta)?.parse()?);
        } else {
            return Err(meta.error("unknown field attribute"));
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Variant {
    pub with: Option<Path>,
}

impl Variant {
    pub fn parse(attributes: &[Attribute]) -> Result<Self> {
        let mut variant = Self::default();
        for attribute in attributes {
            if !attribute.path().is_ident("uklad") {
                continue;
            }
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("with") {
                    if variant.with.is_some() {
                        return Err(meta.error("duplicate `with` attribute"));
                    }
                    variant.with = Some(literal(&meta)?.parse()?);
                    Ok(())
                } else {
                    Err(meta.error("unknown variant attribute"))
                }
            })?;
        }
        Ok(variant)
    }
}

fn literal(meta: &ParseNestedMeta<'_>) -> Result<LitStr> {
    meta.value()?.parse()
}

fn predicates(meta: &ParseNestedMeta<'_>) -> Result<Punctuated<WherePredicate, Token![,]>> {
    literal(meta)?.parse_with(Punctuated::parse_terminated)
}
