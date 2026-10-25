use super::{Encoder, Sealed, write::Writer};
use crate::config::Config;

/// An encoder that writes to a [`Writer`].
///
/// Most code should use the top-level `encode_*` functions instead.
///
/// ```
/// use uklad::enc::{EncoderImpl, write::SliceWriter};
///
/// let mut buffer = [0u8; 4];
/// let mut encoder = EncoderImpl::new(
///     SliceWriter::new(&mut buffer),
///     uklad::config::legacy().with_big_endian(),
/// );
///
/// uklad::Encode::encode(&5u32, &mut encoder).unwrap();
/// assert_eq!(buffer, [0, 0, 0, 5]);
/// ```
pub struct EncoderImpl<W: Writer, C: Config> {
    writer: W,
    config: C,
}

impl<W: Writer, C: Config> EncoderImpl<W, C> {
    /// Creates an encoder writing to `writer`.
    pub const fn new(writer: W, config: C) -> Self {
        Self { writer, config }
    }

    /// Returns the underlying writer.
    #[inline]
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W: Writer, C: Config> Sealed for EncoderImpl<W, C> {}

impl<W: Writer, C: Config> Encoder for EncoderImpl<W, C> {
    type W = W;
    type C = C;

    #[inline]
    fn writer(&mut self) -> &mut Self::W {
        &mut self.writer
    }

    #[inline]
    fn config(&self) -> &Self::C {
        &self.config
    }
}
