use super::{
    BorrowDecode, BorrowDecoder, Decode, Decoder,
    read::{BorrowReader, Reader},
};
use crate::{
    config::{Config, Endianness, IntEncoding},
    error::{DecodeError, IntegerType},
    varint::{self, MaxWidth},
};

use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};

fn fixed_to_u128(bytes: &[u8], endian: Endianness) -> u128 {
    let mut value = 0u128;
    match endian {
        Endianness::Big => {
            for &byte in bytes {
                value = (value << 8) | byte as u128;
            }
        }
        Endianness::Little => {
            for &byte in bytes.iter().rev() {
                value = (value << 8) | byte as u128;
            }
        }
    }
    value
}

impl<Context> Decode<Context> for bool {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u8::decode(decoder)? {
            0 => Ok(false),
            1 => Ok(true),
            found => Err(DecodeError::InvalidBooleanValue(found)),
        }
    }
}
crate::impl_borrow_decode!(bool);

impl<Context> Decode<Context> for u8 {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(1)?;
        if let Some(byte) = decoder.reader().peek_read(1) {
            let byte = byte[0];
            decoder.reader().consume(1);
            return Ok(byte);
        }
        let mut byte = [0u8; 1];
        decoder.reader().read(&mut byte)?;
        Ok(byte[0])
    }
}
crate::impl_borrow_decode!(u8);

impl<Context> Decode<Context> for i8 {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(u8::decode(decoder)? as i8)
    }
}
crate::impl_borrow_decode!(i8);

macro_rules! impl_unsigned_decode {
    ($ty:ty, $size:expr, $integer:ident, $max:ident) => {
        impl<Context> Decode<Context> for $ty {
            fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
                decoder.claim_bytes_read($size)?;
                let value = match D::C::INT_ENCODING {
                    IntEncoding::Variable => varint::decode_unsigned(
                        decoder.reader(),
                        D::C::ENDIANNESS,
                        IntegerType::$integer,
                        MaxWidth::$max,
                    )?,
                    IntEncoding::Fixed => {
                        let mut bytes = [0u8; $size];
                        decoder.reader().read(&mut bytes)?;
                        fixed_to_u128(&bytes, D::C::ENDIANNESS)
                    }
                };
                Ok(value as $ty)
            }
        }
        crate::impl_borrow_decode!($ty);
    };
}

impl_unsigned_decode!(u16, 2, U16, U16);
impl_unsigned_decode!(u32, 4, U32, U32);
impl_unsigned_decode!(u64, 8, U64, U64);
impl_unsigned_decode!(u128, 16, U128, U128);

macro_rules! impl_signed_decode {
    ($ty:ty, $size:expr, $integer:ident, $max:ident) => {
        impl<Context> Decode<Context> for $ty {
            fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
                decoder.claim_bytes_read($size)?;
                let value = match D::C::INT_ENCODING {
                    IntEncoding::Variable => varint::decode_unsigned(
                        decoder.reader(),
                        D::C::ENDIANNESS,
                        IntegerType::$integer,
                        MaxWidth::$max,
                    )
                    .map_err(DecodeError::into_signed)
                    .map(varint::zigzag_decode)?,
                    IntEncoding::Fixed => {
                        let mut bytes = [0u8; $size];
                        decoder.reader().read(&mut bytes)?;
                        fixed_to_u128(&bytes, D::C::ENDIANNESS) as i128
                    }
                };
                Ok(value as $ty)
            }
        }
        crate::impl_borrow_decode!($ty);
    };
}

impl_signed_decode!(i16, 2, I16, U16);
impl_signed_decode!(i32, 4, I32, U32);
impl_signed_decode!(i64, 8, I64, U64);
impl_signed_decode!(i128, 16, I128, U128);

impl<Context> Decode<Context> for usize {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(8)?;
        let value = match D::C::INT_ENCODING {
            IntEncoding::Variable => varint::decode_unsigned(
                decoder.reader(),
                D::C::ENDIANNESS,
                IntegerType::Usize,
                MaxWidth::Usize,
            )? as u64,
            IntEncoding::Fixed => {
                let mut bytes = [0u8; 8];
                decoder.reader().read(&mut bytes)?;
                fixed_to_u128(&bytes, D::C::ENDIANNESS) as u64
            }
        };
        usize::try_from(value).map_err(|_| DecodeError::OutsideUsizeRange(value))
    }
}
crate::impl_borrow_decode!(usize);

impl<Context> Decode<Context> for isize {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(i64::decode(decoder)? as isize)
    }
}
crate::impl_borrow_decode!(isize);

macro_rules! impl_nonzero_decode {
    ($ty:ty, $inner:ty, $integer:ident) => {
        impl<Context> Decode<Context> for $ty {
            fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
                <$ty>::new(<$inner>::decode(decoder)?).ok_or(DecodeError::NonZeroTypeIsZero {
                    non_zero_type: IntegerType::$integer,
                })
            }
        }
        crate::impl_borrow_decode!($ty);
    };
}

impl_nonzero_decode!(NonZeroU8, u8, U8);
impl_nonzero_decode!(NonZeroU16, u16, U16);
impl_nonzero_decode!(NonZeroU32, u32, U32);
impl_nonzero_decode!(NonZeroU64, u64, U64);
impl_nonzero_decode!(NonZeroU128, u128, U128);
impl_nonzero_decode!(NonZeroUsize, usize, Usize);
impl_nonzero_decode!(NonZeroI8, i8, I8);
impl_nonzero_decode!(NonZeroI16, i16, I16);
impl_nonzero_decode!(NonZeroI32, i32, I32);
impl_nonzero_decode!(NonZeroI64, i64, I64);
impl_nonzero_decode!(NonZeroI128, i128, I128);
impl_nonzero_decode!(NonZeroIsize, isize, Isize);

impl<Context> Decode<Context> for f32 {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(4)?;
        let mut bytes = [0u8; 4];
        decoder.reader().read(&mut bytes)?;
        Ok(match D::C::ENDIANNESS {
            Endianness::Big => f32::from_be_bytes(bytes),
            Endianness::Little => f32::from_le_bytes(bytes),
        })
    }
}
crate::impl_borrow_decode!(f32);

impl<Context> Decode<Context> for f64 {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(8)?;
        let mut bytes = [0u8; 8];
        decoder.reader().read(&mut bytes)?;
        Ok(match D::C::ENDIANNESS {
            Endianness::Big => f64::from_be_bytes(bytes),
            Endianness::Little => f64::from_le_bytes(bytes),
        })
    }
}
crate::impl_borrow_decode!(f64);

impl<Context> Decode<Context> for char {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let mut bytes = [0u8; 4];
        decoder.reader().read(&mut bytes[..1])?;
        let width = utf8_width(bytes[0]);
        if width == 0 {
            return Err(DecodeError::InvalidCharEncoding(bytes));
        }
        decoder.claim_bytes_read(width)?;
        if width == 1 {
            return Ok(bytes[0] as char);
        }
        decoder.reader().read(&mut bytes[1..width])?;
        core::str::from_utf8(&bytes[..width])
            .ok()
            .and_then(|s| s.chars().next())
            .ok_or(DecodeError::InvalidCharEncoding(bytes))
    }
}
crate::impl_borrow_decode!(char);

const fn utf8_width(byte: u8) -> usize {
    match byte {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => 0,
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for &'de [u8] {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let length = super::decode_slice_len(decoder)?;
        decoder.claim_bytes_read(length)?;
        decoder.borrow_reader().take_bytes(length)
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for &'de str {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let bytes = <&[u8]>::borrow_decode(decoder)?;
        core::str::from_utf8(bytes).map_err(|inner| DecodeError::Utf8 { inner })
    }
}
