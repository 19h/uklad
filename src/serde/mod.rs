//! Support for serde integration. Enable this with the `serde` feature.
//!
//! To encode/decode type that implement serde's trait, you can use:
//! - [borrow_decode_from_slice]
//! - [decode_from_slice]
//! - [encode_into_slice]
//! - [encode_to_vec]
//!
//! For interop with uklad's [Decode]/[Encode], you can use:
//! - [Compat]
//! - [BorrowCompat]
//!
//! For interop with uklad's `derive` feature, you can use the `#[uklad(with_serde)]` attribute on each field that implements serde's traits.
//!
//! ```
//! # #[cfg(feature = "derive")]
//! # mod foo {
//! # use uklad::{Decode, Encode};
//! # use serde::{Deserialize, Serialize};
//! #[derive(Serialize, Deserialize)]
//! pub struct SerdeType {
//!     // ...
//! }
//!
//! #[derive(Decode, Encode)]
//! pub struct StructWithSerde {
//!     #[uklad(with_serde)]
//!     pub serde: SerdeType,
//! }
//!
//! #[derive(Decode, Encode)]
//! pub enum EnumWithSerde {
//!     Unit(#[uklad(with_serde)] SerdeType),
//!     Struct {
//!         #[uklad(with_serde)]
//!         serde: SerdeType,
//!     },
//! }
//! # }
//! ```
//!
//! # Known issues
//!
//! Because uklad is a format without metadata, several of serde's attributes are unsupported. Avoid the following attributes with uklad, or use uklad's own `derive` macros instead.
//! - `#[serde(flatten)]`
//! - `#[serde(skip)]`
//! - `#[serde(skip_deserializing)]`
//! - `#[serde(skip_serializing)]`
//! - `#[serde(skip_serializing_if = "path")]`
//! - `#[serde(tag = "...")]`
//! - `#[serde(untagged)]`
//!
//! **These attributes are not supported and can cause data loss.** Use uklad's own derive macros instead.
//!
//! # Relationship with serde
//!
//! Serde remains fully supported; it is an optional dependency so that projects can choose their own dependency set. With all features disabled, uklad has no dependencies.
//!
//! Reasons to prefer uklad's own derive macros:
//! - The attribute limitations documented above.
//! - Serde does not define an MSRV ([source](https://github.com/serde-rs/serde/pull/2257)); this crate treats its MSRV as a compatibility guarantee.
//! - Fewer dependencies means a smaller supply-chain surface.
//!
//! [Decode]: ../de/trait.Decode.html
//! [Encode]: ../enc/trait.Encode.html

mod de_borrowed;
mod de_owned;
mod ser;

pub use self::de_borrowed::*;
pub use self::de_owned::*;
pub use self::ser::*;

/// A serde-specific error that occurred while decoding.
#[derive(Debug)]
#[non_exhaustive]
pub enum DecodeError {
    /// Uklad does not support serde's `any` decoding feature.
    ///
    /// See the "known issues" list in the serde module for more information on this.
    AnyNotSupported,

    /// Uklad does not support serde identifiers
    IdentifierNotSupported,

    /// Uklad does not support serde's `ignored_any`.
    ///
    /// See the "known issues" list in the serde module for more information on this.
    IgnoredAnyNotSupported,

    /// Serde tried decoding a borrowed value from an owned reader. Use `serde_decode_borrowed_from_*` instead
    CannotBorrowOwnedData,

    /// Could not allocate data like `String` and `Vec<u8>`
    #[cfg(not(feature = "alloc"))]
    CannotAllocate,

    /// Custom serde error but uklad is unable to allocate a string. Set a breakpoint where this is thrown for more information.
    #[cfg(not(feature = "alloc"))]
    CustomError,
}

#[cfg(feature = "alloc")]
impl serde::de::Error for crate::error::DecodeError {
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        use alloc::string::ToString;
        Self::OtherString(msg.to_string())
    }
}

#[cfg(not(feature = "alloc"))]
impl serde::de::Error for crate::error::DecodeError {
    fn custom<T>(_: T) -> Self
    where
        T: core::fmt::Display,
    {
        DecodeError::CustomError.into()
    }
}

#[allow(clippy::from_over_into)]
impl Into<crate::error::DecodeError> for DecodeError {
    fn into(self) -> crate::error::DecodeError {
        crate::error::DecodeError::Serde(self)
    }
}

/// A serde-specific error that occurred while encoding.
#[derive(Debug)]
#[non_exhaustive]
pub enum EncodeError {
    /// Serde provided uklad with a sequence without a length, which is not supported in uklad
    SequenceMustHaveLength,

    /// [Serializer::collect_str] got called but uklad was unable to allocate memory.
    #[cfg(not(feature = "alloc"))]
    CannotCollectStr,

    /// Custom serde error but uklad is unable to allocate a string. Set a breakpoint where this is thrown for more information.
    #[cfg(not(feature = "alloc"))]
    CustomError,
}

#[allow(clippy::from_over_into)]
impl Into<crate::error::EncodeError> for EncodeError {
    fn into(self) -> crate::error::EncodeError {
        crate::error::EncodeError::Serde(self)
    }
}

#[cfg(feature = "alloc")]
impl serde::ser::Error for crate::error::EncodeError {
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        use alloc::string::ToString;

        Self::OtherString(msg.to_string())
    }
}

#[cfg(not(feature = "alloc"))]
impl serde::ser::Error for crate::error::EncodeError {
    fn custom<T>(_: T) -> Self
    where
        T: core::fmt::Display,
    {
        EncodeError::CustomError.into()
    }
}

/// Wrapper struct that implements [Decode] and [Encode] on any type that implements serde's [DeserializeOwned] and [Serialize] respectively.
///
/// This works for most types, but if you're dealing with borrowed data consider using [BorrowCompat] instead.
///
/// [Decode]: ../de/trait.Decode.html
/// [Encode]: ../enc/trait.Encode.html
/// [DeserializeOwned]: https://docs.rs/serde/1/serde/de/trait.DeserializeOwned.html
/// [Serialize]: https://docs.rs/serde/1/serde/trait.Serialize.html
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Compat<T>(pub T);

impl<Context, T> crate::Decode<Context> for Compat<T>
where
    T: serde::de::DeserializeOwned,
{
    fn decode<D: crate::de::Decoder>(decoder: &mut D) -> Result<Self, crate::error::DecodeError> {
        let serde_decoder = de_owned::SerdeDecoder { de: decoder };
        T::deserialize(serde_decoder).map(Compat)
    }
}
impl<'de, T, Context> crate::BorrowDecode<'de, Context> for Compat<T>
where
    T: serde::de::DeserializeOwned,
{
    fn borrow_decode<D: crate::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, crate::error::DecodeError> {
        let serde_decoder = de_owned::SerdeDecoder { de: decoder };
        T::deserialize(serde_decoder).map(Compat)
    }
}

impl<T> crate::Encode for Compat<T>
where
    T: serde::Serialize,
{
    fn encode<E: crate::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), crate::error::EncodeError> {
        let serializer = ser::SerdeEncoder { enc: encoder };
        self.0.serialize(serializer)?;
        Ok(())
    }
}

impl<T> core::fmt::Debug for Compat<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Compat").field(&self.0).finish()
    }
}

impl<T> core::fmt::Display for Compat<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

/// Wrapper struct that implements [BorrowDecode] and [Encode] on any type that implements serde's [Deserialize] and [Serialize] respectively. This is mostly used on `&[u8]` and `&str`, for other types consider using [Compat] instead.
///
/// [BorrowDecode]: ../de/trait.BorrowDecode.html
/// [Encode]: ../enc/trait.Encode.html
/// [Deserialize]: https://docs.rs/serde/1/serde/de/trait.Deserialize.html
/// [Serialize]: https://docs.rs/serde/1/serde/trait.Serialize.html
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BorrowCompat<T>(pub T);

impl<'de, T, Context> crate::de::BorrowDecode<'de, Context> for BorrowCompat<T>
where
    T: serde::de::Deserialize<'de>,
{
    fn borrow_decode<D: crate::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, crate::error::DecodeError> {
        let serde_decoder = de_borrowed::SerdeDecoder {
            de: decoder,
            pd: core::marker::PhantomData,
        };
        T::deserialize(serde_decoder).map(BorrowCompat)
    }
}

impl<T> crate::Encode for BorrowCompat<T>
where
    T: serde::Serialize,
{
    fn encode<E: crate::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), crate::error::EncodeError> {
        let serializer = ser::SerdeEncoder { enc: encoder };
        self.0.serialize(serializer)?;
        Ok(())
    }
}

impl<T> core::fmt::Debug for BorrowCompat<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("BorrowCompat").field(&self.0).finish()
    }
}

impl<T> core::fmt::Display for BorrowCompat<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
