//! Decoding traits and types.

pub mod read;

mod compound;
mod decoder;
mod primitives;
mod tuples;

pub use decoder::{DecoderImpl, WithContext};

use self::read::{BorrowReader, Reader};
use crate::{config::Config, error::DecodeError};

#[doc(hidden)]
pub trait Sealed {}

impl<T: Sealed + ?Sized> Sealed for &mut T {}

/// A value that can be decoded from an owned reader.
///
/// Types containing borrowed data should implement [`BorrowDecode`] instead.
pub trait Decode<Context>: Sized {
    /// Decodes a value with `decoder`.
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError>;
}

/// A value that can be decoded with borrowed data.
pub trait BorrowDecode<'de, Context>: Sized {
    /// Decodes a value with `decoder`.
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError>;
}

/// Implements [`BorrowDecode`] for a type that implements [`Decode`].
#[macro_export]
macro_rules! impl_borrow_decode {
    ($ty:ty $(, $($params:tt)*)?) => {
        $crate::impl_borrow_decode_with_context!($ty, __Context, __Context $(, $($params)*)?);
    };
}

/// Implements [`BorrowDecode`] with a specific context for a type that
/// implements [`Decode`].
#[macro_export]
macro_rules! impl_borrow_decode_with_context {
    ($ty:ty, $context:ty $(, $($params:tt)*)?) => {
        impl<'de $(, $($params)*)?> $crate::BorrowDecode<'de, $context> for $ty {
            fn borrow_decode<D: $crate::de::BorrowDecoder<'de, Context = $context>>(
                decoder: &mut D,
            ) -> core::result::Result<Self, $crate::error::DecodeError> {
                <Self as $crate::Decode<$context>>::decode(decoder)
            }
        }
    };
}

/// The source of encoded bytes.
///
/// This trait is implemented by [`read::SliceReader`] and is not intended to be
/// implemented by users.
pub trait Decoder: Sealed {
    /// The reader used by this decoder.
    type R: Reader;

    /// The configuration used by this decoder.
    type C: Config;

    /// The decoding context.
    type Context;

    /// Returns the decoding context.
    fn context(&mut self) -> &mut Self::Context;

    /// Returns a decoder that uses `context`.
    fn with_context<C>(&mut self, context: C) -> WithContext<'_, Self, C> {
        WithContext {
            decoder: self,
            context,
        }
    }

    /// Returns the underlying reader.
    fn reader(&mut self) -> &mut Self::R;

    /// Returns the configuration.
    fn config(&self) -> &Self::C;

    /// Declares that `n` bytes will be read, checking the configured limit.
    fn claim_bytes_read(&mut self, n: usize) -> Result<(), DecodeError>;

    /// Declares that a container with `len` elements of `T` will be read.
    fn claim_container_read<T>(&mut self, len: usize) -> Result<(), DecodeError> {
        if Self::C::LIMIT.is_some() {
            match len.checked_mul(core::mem::size_of::<T>()) {
                Some(bytes) => self.claim_bytes_read(bytes),
                None => Err(DecodeError::LimitExceeded),
            }
        } else {
            Ok(())
        }
    }

    /// Releases a previously claimed byte count.
    ///
    /// Container implementations claim the space for their whole contents up
    /// front, then release the space for each element as it is decoded.
    fn unclaim_bytes_read(&mut self, n: usize);
}

/// A [`Decoder`] that can also return borrowed data.
pub trait BorrowDecoder<'de>: Decoder {
    /// The borrow-capable reader used by this decoder.
    type BR: BorrowReader<'de>;

    /// Returns the borrow-capable reader.
    fn borrow_reader(&mut self) -> &mut Self::BR;
}

impl<T: Decoder + ?Sized> Decoder for &mut T {
    type R = T::R;
    type C = T::C;
    type Context = T::Context;

    #[inline]
    fn context(&mut self) -> &mut Self::Context {
        (**self).context()
    }

    #[inline]
    fn reader(&mut self) -> &mut Self::R {
        (**self).reader()
    }

    #[inline]
    fn config(&self) -> &Self::C {
        (**self).config()
    }

    #[inline]
    fn claim_bytes_read(&mut self, n: usize) -> Result<(), DecodeError> {
        (**self).claim_bytes_read(n)
    }

    #[inline]
    fn unclaim_bytes_read(&mut self, n: usize) {
        (**self).unclaim_bytes_read(n)
    }
}

impl<'de, T: BorrowDecoder<'de> + ?Sized> BorrowDecoder<'de> for &mut T {
    type BR = T::BR;

    #[inline]
    fn borrow_reader(&mut self) -> &mut Self::BR {
        (**self).borrow_reader()
    }
}

#[inline]
pub(crate) fn decode_option_variant<D: Decoder>(
    decoder: &mut D,
    type_name: &'static str,
) -> Result<Option<()>, DecodeError> {
    match u8::decode(decoder)? {
        0 => Ok(None),
        1 => Ok(Some(())),
        found => Err(DecodeError::UnexpectedVariant {
            found: found as u32,
            allowed: &crate::error::AllowedEnumVariants::Range { min: 0, max: 1 },
            type_name,
        }),
    }
}

#[inline]
pub(crate) fn decode_slice_len<D: Decoder>(decoder: &mut D) -> Result<usize, DecodeError> {
    let value = u64::decode(decoder)?;
    value
        .try_into()
        .map_err(|_| DecodeError::OutsideUsizeRange(value))
}
