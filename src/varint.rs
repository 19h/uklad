//! Variable-length integer encoding.
//!
//! Layout:
//!
//! - `0..=250`: the value itself.
//! - `251`: marker followed by a `u16`.
//! - `252`: marker followed by a `u32`.
//! - `253`: marker followed by a `u64`.
//! - `254`: marker followed by a `u128`.
//! - `255`: reserved.

use crate::{
    config::{Endianness, U16_MARKER, U32_MARKER, U64_MARKER, U128_MARKER},
    de::read::Reader,
    enc::write::Writer,
    error::{DecodeError, EncodeError, IntegerType},
};

const RESERVED: u8 = 255;
const INLINE_MAX: u128 = 250;

/// The maximum integer width accepted by a target type.
#[derive(Copy, Clone)]
pub(crate) enum MaxWidth {
    U16,
    U32,
    U64,
    U128,
    Usize,
}

impl MaxWidth {
    const fn bytes(self) -> usize {
        match self {
            Self::U16 => 2,
            Self::U32 => 4,
            Self::U64 | Self::Usize => 8,
            Self::U128 => 16,
        }
    }
}

pub(crate) fn encode<W: Writer>(
    writer: &mut W,
    endian: Endianness,
    value: u128,
) -> Result<(), EncodeError> {
    let mut buffer = [0u8; 17];
    let (length, payload) = if value <= INLINE_MAX {
        buffer[0] = value as u8;
        (1, 0)
    } else if value <= u16::MAX as u128 {
        buffer[0] = U16_MARKER;
        (3, 2)
    } else if value <= u32::MAX as u128 {
        buffer[0] = U32_MARKER;
        (5, 4)
    } else if value <= u64::MAX as u128 {
        buffer[0] = U64_MARKER;
        (9, 8)
    } else {
        buffer[0] = U128_MARKER;
        (17, 16)
    };

    if payload != 0 {
        let bytes = value.to_be_bytes();
        let source = &bytes[16 - payload..];
        match endian {
            Endianness::Big => buffer[1..1 + payload].copy_from_slice(source),
            Endianness::Little => {
                for (dst, src) in buffer[1..1 + payload].iter_mut().zip(source.iter().rev()) {
                    *dst = *src;
                }
            }
        }
    }

    writer.write(&buffer[..length])
}

pub(crate) fn decode_unsigned<R: Reader>(
    reader: &mut R,
    endian: Endianness,
    expected: IntegerType,
    max: MaxWidth,
) -> Result<u128, DecodeError> {
    let Some(first) = reader.peek_read(1) else {
        let mut first = [0u8; 1];
        reader.read(&mut first)?;
        return read_payload(reader, endian, expected, max, first[0]);
    };

    let tag = first[0];
    if tag <= 250 {
        reader.consume(1);
        return Ok(tag as u128);
    }

    let (payload, found) = header(tag).ok_or(DecodeError::InvalidIntegerType {
        expected,
        found: IntegerType::Reserved,
    })?;
    ensure_fits(max, expected, found)?;

    if let Some(buffer) = reader.peek_read(1 + payload) {
        let value = decode_bytes(&buffer[1..], endian);
        reader.consume(1 + payload);
        return Ok(value);
    }

    reader.consume(1);
    let mut buffer = [0u8; 16];
    reader.read(&mut buffer[..payload])?;
    Ok(decode_bytes(&buffer[..payload], endian))
}

fn read_payload<R: Reader>(
    reader: &mut R,
    endian: Endianness,
    expected: IntegerType,
    max: MaxWidth,
    tag: u8,
) -> Result<u128, DecodeError> {
    if tag <= 250 {
        return Ok(tag as u128);
    }

    let (payload, found) = header(tag).ok_or(DecodeError::InvalidIntegerType {
        expected,
        found: IntegerType::Reserved,
    })?;
    ensure_fits(max, expected, found)?;
    let mut buffer = [0u8; 16];
    reader.read(&mut buffer[..payload])?;
    Ok(decode_bytes(&buffer[..payload], endian))
}

fn header(tag: u8) -> Option<(usize, IntegerType)> {
    match tag {
        U16_MARKER => Some((2, IntegerType::U16)),
        U32_MARKER => Some((4, IntegerType::U32)),
        U64_MARKER => Some((8, IntegerType::U64)),
        U128_MARKER => Some((16, IntegerType::U128)),
        RESERVED => None,
        _ => unreachable!("inline values are handled before the header"),
    }
}

const fn ensure_fits(
    max: MaxWidth,
    expected: IntegerType,
    found: IntegerType,
) -> Result<(), DecodeError> {
    if max.bytes() >= payload_width(found) {
        Ok(())
    } else {
        Err(DecodeError::InvalidIntegerType { expected, found })
    }
}

const fn payload_width(found: IntegerType) -> usize {
    match found {
        IntegerType::U16 => 2,
        IntegerType::U32 => 4,
        IntegerType::U64 => 8,
        IntegerType::U128 => 16,
        _ => 1,
    }
}

fn decode_bytes(bytes: &[u8], endian: Endianness) -> u128 {
    let mut value: u128 = 0;
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

pub(crate) fn zigzag_encode(value: i128) -> u128 {
    ((value << 1) ^ (value >> 127)) as u128
}

pub(crate) fn zigzag_decode(value: u128) -> i128 {
    ((value >> 1) as i128) ^ -((value & 1) as i128)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enc::write::SliceWriter;

    fn encode_value(value: u128, endian: Endianness) -> std::vec::Vec<u8> {
        let mut buffer = [0u8; 17];
        let length = {
            let mut writer = SliceWriter::new(&mut buffer);
            encode(&mut writer, endian, value).unwrap();
            writer.bytes_written()
        };
        buffer[..length].to_vec()
    }

    #[test]
    fn encodes_minimal_width() {
        assert_eq!(encode_value(0, Endianness::Little), [0]);
        assert_eq!(encode_value(250, Endianness::Little), [250]);
        assert_eq!(encode_value(251, Endianness::Little), [251, 251, 0]);
        assert_eq!(encode_value(65535, Endianness::Big), [251, 255, 255]);
        assert_eq!(encode_value(65536, Endianness::Big), [252, 0, 1, 0, 0]);
        assert_eq!(encode_value(u64::MAX as u128, Endianness::Big)[0], 253);
        assert_eq!(encode_value(u64::MAX as u128 + 1, Endianness::Big)[0], 254);
    }

    #[test]
    fn zigzag_round_trips() {
        for value in [i128::MIN, -1000, -1, 0, 1, 1000, i128::MAX] {
            assert_eq!(zigzag_decode(zigzag_encode(value)), value);
        }
    }
}
