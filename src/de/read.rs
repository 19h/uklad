//! Byte sources used by the decoder.
//!
//! [`Reader`] reads owned data. [`BorrowReader`] additionally supports returning
//! slices borrowed from the input.

use crate::error::DecodeError;

/// A source of encoded bytes.
pub trait Reader {
    /// Fills `bytes` entirely, or returns an error.
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError>;

    /// Returns up to `n` bytes without consuming them, if the reader supports
    /// peeking.
    #[inline]
    fn peek_read(&mut self, _n: usize) -> Option<&[u8]> {
        None
    }

    /// Consumes `n` bytes previously observed with [`Reader::peek_read`].
    #[inline]
    fn consume(&mut self, _n: usize) {}
}

impl<T: Reader + ?Sized> Reader for &mut T {
    #[inline]
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        (**self).read(bytes)
    }

    #[inline]
    fn peek_read(&mut self, n: usize) -> Option<&[u8]> {
        (**self).peek_read(n)
    }

    #[inline]
    fn consume(&mut self, n: usize) {
        (**self).consume(n);
    }
}

/// A reader that can return slices borrowed from its input.
pub trait BorrowReader<'storage>: Reader {
    /// Returns exactly `length` bytes and advances the reader.
    fn take_bytes(&mut self, length: usize) -> Result<&'storage [u8], DecodeError>;
}

#[doc(hidden)]
pub trait Position {
    fn position(&self) -> usize;
}

/// A reader over a byte slice.
pub struct SliceReader<'a> {
    pub(crate) slice: &'a [u8],
    original_len: usize,
}

impl<'a> SliceReader<'a> {
    /// Creates a reader over `slice`.
    pub const fn new(slice: &'a [u8]) -> Self {
        Self {
            slice,
            original_len: slice.len(),
        }
    }
}

impl Position for SliceReader<'_> {
    #[inline]
    fn position(&self) -> usize {
        self.original_len - self.slice.len()
    }
}

impl Reader for SliceReader<'_> {
    #[inline]
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        if bytes.len() > self.slice.len() {
            return Err(DecodeError::UnexpectedEnd {
                additional: bytes.len() - self.slice.len(),
            });
        }
        let (head, tail) = self.slice.split_at(bytes.len());
        bytes.copy_from_slice(head);
        self.slice = tail;
        Ok(())
    }

    #[inline]
    fn peek_read(&mut self, n: usize) -> Option<&[u8]> {
        self.slice.get(..n)
    }

    #[inline]
    fn consume(&mut self, n: usize) {
        self.slice = self.slice.get(n..).unwrap_or_default();
    }
}

impl<'a> BorrowReader<'a> for SliceReader<'a> {
    #[inline]
    fn take_bytes(&mut self, length: usize) -> Result<&'a [u8], DecodeError> {
        if length > self.slice.len() {
            return Err(DecodeError::UnexpectedEnd {
                additional: length - self.slice.len(),
            });
        }
        let (head, tail) = self.slice.split_at(length);
        self.slice = tail;
        Ok(head)
    }
}
