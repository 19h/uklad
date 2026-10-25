//! Derive macros for the `uklad` crate.
//!
//! The macros implement `Encode`, `Decode`, and `BorrowDecode`. Field and
//! container options are provided through `#[uklad(...)]` attributes.

mod attrs;
mod expand;

use proc_macro::TokenStream;

/// Derives `uklad::Encode`.
#[proc_macro_derive(Encode, attributes(uklad))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    expand::encode(syn::parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives `uklad::Decode` and a matching `uklad::BorrowDecode` implementation.
#[proc_macro_derive(Decode, attributes(uklad))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    expand::decode(syn::parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives `uklad::BorrowDecode` for a type that contains borrowed data.
#[proc_macro_derive(BorrowDecode, attributes(uklad))]
pub fn derive_borrow_decode(input: TokenStream) -> TokenStream {
    expand::borrow_decode(syn::parse_macro_input!(input))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
