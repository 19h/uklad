use super::{
    BorrowDecoder, Decoder, Sealed,
    read::{BorrowReader, Reader},
};
use crate::{config::Config, error::DecodeError};

/// A decoder that reads from a [`Reader`].
///
/// Most code should use the top-level `decode_*` functions instead.
pub struct DecoderImpl<R, C: Config, Context> {
    reader: R,
    config: C,
    claimed: usize,
    context: Context,
}

impl<R: Reader, C: Config, Context> DecoderImpl<R, C, Context> {
    /// Creates a decoder reading from `reader`.
    pub const fn new(reader: R, config: C, context: Context) -> Self {
        Self {
            reader,
            config,
            claimed: 0,
            context,
        }
    }

    /// Returns the number of bytes read when the reader is a slice reader.
    #[inline]
    pub fn bytes_read(&self) -> usize
    where
        R: super::read::Position,
    {
        self.reader.position()
    }
}

impl<R, C: Config, Context> Sealed for DecoderImpl<R, C, Context> {}

impl<'de, R: BorrowReader<'de>, C: Config, Context> BorrowDecoder<'de>
    for DecoderImpl<R, C, Context>
{
    type BR = R;

    #[inline]
    fn borrow_reader(&mut self) -> &mut Self::BR {
        &mut self.reader
    }
}

impl<R: Reader, C: Config, Context> Decoder for DecoderImpl<R, C, Context> {
    type R = R;
    type C = C;
    type Context = Context;

    #[inline]
    fn context(&mut self) -> &mut Self::Context {
        &mut self.context
    }

    #[inline]
    fn reader(&mut self) -> &mut Self::R {
        &mut self.reader
    }

    #[inline]
    fn config(&self) -> &Self::C {
        &self.config
    }

    #[inline]
    fn claim_bytes_read(&mut self, n: usize) -> Result<(), DecodeError> {
        if let Some(limit) = C::LIMIT {
            self.claimed = self
                .claimed
                .checked_add(n)
                .ok_or(DecodeError::LimitExceeded)?;
            if self.claimed > limit {
                return Err(DecodeError::LimitExceeded);
            }
        }
        Ok(())
    }

    #[inline]
    fn unclaim_bytes_read(&mut self, n: usize) {
        if C::LIMIT.is_some() {
            self.claimed -= n;
        }
    }
}

/// A decoder wrapper that supplies a context.
pub struct WithContext<'a, D: ?Sized, C> {
    pub(crate) decoder: &'a mut D,
    pub(crate) context: C,
}

impl<D: Decoder + ?Sized, C> Sealed for WithContext<'_, D, C> {}

impl<D: Decoder + ?Sized, Context> Decoder for WithContext<'_, D, Context> {
    type R = D::R;
    type C = D::C;
    type Context = Context;

    #[inline]
    fn context(&mut self) -> &mut Self::Context {
        &mut self.context
    }

    #[inline]
    fn reader(&mut self) -> &mut Self::R {
        self.decoder.reader()
    }

    #[inline]
    fn config(&self) -> &Self::C {
        self.decoder.config()
    }

    #[inline]
    fn claim_bytes_read(&mut self, n: usize) -> Result<(), DecodeError> {
        self.decoder.claim_bytes_read(n)
    }

    #[inline]
    fn unclaim_bytes_read(&mut self, n: usize) {
        self.decoder.unclaim_bytes_read(n)
    }
}

impl<'de, D: BorrowDecoder<'de>, C> BorrowDecoder<'de> for WithContext<'_, D, C> {
    type BR = D::BR;

    #[inline]
    fn borrow_reader(&mut self) -> &mut Self::BR {
        self.decoder.borrow_reader()
    }
}
