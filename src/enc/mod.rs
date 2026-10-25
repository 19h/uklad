//! Encoding traits and types.

pub mod write;

mod compound;
mod encoder;
mod primitives;
mod tuples;

pub use encoder::EncoderImpl;

use crate::{config::Config, error::EncodeError};

#[doc(hidden)]
pub trait Sealed {}

impl<T: Sealed + ?Sized> Sealed for &mut T {}

/// A value that can be encoded.
///
/// Implement this trait directly, or derive it with the `derive` feature:
///
/// ```
/// # #[cfg(feature = "derive")]
/// # mod example {
/// #[derive(uklad::Encode)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// # }
/// ```
pub trait Encode {
    /// Encodes `self` with `encoder`.
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError>;
}

/// The destination of encoded values.
///
/// This trait is implemented by [`EncoderImpl`] and is not intended to be
/// implemented directly.
pub trait Encoder: Sealed {
    /// The writer used by this encoder.
    type W: write::Writer;

    /// The configuration used by this encoder.
    type C: Config;

    /// Returns the underlying writer.
    fn writer(&mut self) -> &mut Self::W;

    /// Returns the configuration.
    fn config(&self) -> &Self::C;
}

impl<T: Encoder + ?Sized> Encoder for &mut T {
    type W = T::W;
    type C = T::C;

    #[inline]
    fn writer(&mut self) -> &mut Self::W {
        (**self).writer()
    }

    #[inline]
    fn config(&self) -> &Self::C {
        (**self).config()
    }
}

#[inline]
pub(crate) fn encode_option<E: Encoder>(
    encoder: &mut E,
    value: &Option<()>,
) -> Result<(), EncodeError> {
    match value {
        None => 0u8.encode(encoder),
        Some(()) => 1u8.encode(encoder),
    }
}

#[inline]
pub(crate) fn encode_len<E: Encoder>(encoder: &mut E, len: usize) -> Result<(), EncodeError> {
    (len as u64).encode(encoder)
}
