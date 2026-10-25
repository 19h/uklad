//! Byte-oriented output used by the encoder.

use crate::error::EncodeError;

/// A destination for encoded bytes.
///
/// Implementations must either write the entire slice or return an error.
pub trait Writer {
    /// Writes all of `bytes`.
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError>;
}

impl<T: Writer + ?Sized> Writer for &mut T {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        (**self).write(bytes)
    }
}

/// A writer that fills a pre-allocated slice.
///
/// ```
/// use uklad::enc::write::{SliceWriter, Writer};
///
/// let mut buffer = [0u8; 4];
/// let mut writer = SliceWriter::new(&mut buffer);
/// writer.write(&[1, 2, 3]).unwrap();
///
/// assert_eq!(writer.bytes_written(), 3);
/// assert_eq!(buffer, [1, 2, 3, 0]);
/// ```
pub struct SliceWriter<'a> {
    slice: &'a mut [u8],
    written: usize,
}

impl<'a> SliceWriter<'a> {
    /// Creates a writer over `slice`.
    pub fn new(slice: &'a mut [u8]) -> Self {
        Self { slice, written: 0 }
    }

    /// Returns the number of bytes written.
    pub const fn bytes_written(&self) -> usize {
        self.written
    }
}

impl Writer for SliceWriter<'_> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        let remaining = self.slice.len() - self.written;
        if bytes.len() > remaining {
            return Err(EncodeError::UnexpectedEnd);
        }
        let end = self.written + bytes.len();
        self.slice[self.written..end].copy_from_slice(bytes);
        self.written = end;
        Ok(())
    }
}

/// A writer that only counts bytes.
#[derive(Default)]
pub struct SizeWriter {
    /// The number of bytes written so far.
    pub bytes_written: usize,
}

impl Writer for SizeWriter {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.bytes_written += bytes.len();
        Ok(())
    }
}
