use super::{Encode, Encoder, write::Writer};
use crate::{
    config::{Config, Endianness, IntEncoding},
    error::EncodeError,
    varint,
};

use core::{
    cmp::Reverse,
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize, Wrapping,
    },
};

#[inline]
fn write_fixed<W: super::write::Writer>(
    writer: &mut W,
    endian: Endianness,
    bytes: &[u8],
) -> Result<(), EncodeError> {
    match endian {
        Endianness::Big => writer.write(bytes),
        Endianness::Little => {
            let mut buffer = [0u8; 16];
            let buffer = &mut buffer[..bytes.len()];
            buffer.copy_from_slice(bytes);
            buffer.reverse();
            writer.write(buffer)
        }
    }
}

impl Encode for bool {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self as u8).encode(encoder)
    }
}

impl Encode for u8 {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.writer().write(&[*self])
    }
}

impl Encode for i8 {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.writer().write(&[*self as u8])
    }
}

macro_rules! impl_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Encode for $ty {
                fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
                    match E::C::INT_ENCODING {
                        IntEncoding::Variable => {
                            varint::encode(encoder.writer(), E::C::ENDIANNESS, *self as u128)
                        }
                        IntEncoding::Fixed => write_fixed(
                            encoder.writer(),
                            E::C::ENDIANNESS,
                            &self.to_be_bytes(),
                        ),
                    }
                }
            }
        )*
    };
}

macro_rules! impl_signed {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Encode for $ty {
                fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
                    match E::C::INT_ENCODING {
                        IntEncoding::Variable => varint::encode(
                            encoder.writer(),
                            E::C::ENDIANNESS,
                            varint::zigzag_encode(*self as i128),
                        ),
                        IntEncoding::Fixed => write_fixed(
                            encoder.writer(),
                            E::C::ENDIANNESS,
                            &self.to_be_bytes(),
                        ),
                    }
                }
            }
        )*
    };
}

impl_unsigned!(u16, u32, u64, u128);
impl_signed!(i16, i32, i64, i128);

impl Encode for usize {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self as u64).encode(encoder)
    }
}

impl Encode for isize {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self as i64).encode(encoder)
    }
}

macro_rules! impl_nonzero {
    ($($ty:ty => $inner:ty),* $(,)?) => {
        $(
            impl Encode for $ty {
                #[inline]
                fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
                    self.get().encode(encoder)
                }
            }
        )*
    };
}

impl_nonzero!(
    NonZeroU8 => u8,
    NonZeroU16 => u16,
    NonZeroU32 => u32,
    NonZeroU64 => u64,
    NonZeroU128 => u128,
    NonZeroUsize => usize,
    NonZeroI8 => i8,
    NonZeroI16 => i16,
    NonZeroI32 => i32,
    NonZeroI64 => i64,
    NonZeroI128 => i128,
    NonZeroIsize => isize,
);

impl Encode for f32 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        write_fixed(encoder.writer(), E::C::ENDIANNESS, &self.to_be_bytes())
    }
}

impl Encode for f64 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        write_fixed(encoder.writer(), E::C::ENDIANNESS, &self.to_be_bytes())
    }
}

impl Encode for char {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let mut buffer = [0u8; 4];
        encoder
            .writer()
            .write(self.encode_utf8(&mut buffer).as_bytes())
    }
}

impl<T: Encode> Encode for Wrapping<T> {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}

impl<T: Encode> Encode for Reverse<T> {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode(encoder)
    }
}
