use crate::{
    config::Config,
    de::{BorrowDecode, BorrowDecoder, Decode, Decoder, DecoderImpl, read::Reader},
    enc::{Encode, Encoder, EncoderImpl, write::Writer},
    error::{DecodeError, EncodeError},
};
use core::time::Duration;
use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    hash::{BuildHasher, Hash},
    io::{BufRead, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::{Path, PathBuf},
    string::String,
    sync::{Mutex, RwLock},
    time::SystemTime,
};

/// Decodes a value from a [`std::io::Read`] source.
pub fn decode_from_std_read<D: Decode<()>, C: Config, R: Read>(
    source: &mut R,
    config: C,
) -> Result<D, DecodeError> {
    decode_from_std_read_with_context(source, config, ())
}

/// Decodes a value from a [`std::io::Read`] source with a context.
pub fn decode_from_std_read_with_context<Context, D: Decode<Context>, C: Config, R: Read>(
    source: &mut R,
    config: C,
    context: Context,
) -> Result<D, DecodeError> {
    let mut decoder = DecoderImpl::new(IoReader::new(source), config, context);
    D::decode(&mut decoder)
}

pub(crate) struct IoReader<R> {
    reader: R,
}

impl<R> IoReader<R> {
    pub(crate) const fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: Read> Reader for IoReader<R> {
    #[inline]
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        self.reader
            .read_exact(bytes)
            .map_err(|inner| DecodeError::Io {
                inner,
                additional: bytes.len(),
            })
    }
}

impl<R: Read> Reader for std::io::BufReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        self.read_exact(bytes).map_err(|inner| DecodeError::Io {
            inner,
            additional: bytes.len(),
        })
    }

    #[inline]
    fn peek_read(&mut self, n: usize) -> Option<&[u8]> {
        self.fill_buf().ok().and_then(|buffer| buffer.get(..n))
    }

    #[inline]
    fn consume(&mut self, n: usize) {
        <Self as BufRead>::consume(self, n);
    }
}

/// Encodes a value into a [`std::io::Write`] sink and returns the number of
/// bytes written.
pub fn encode_into_std_write<E: Encode, C: Config, W: Write>(
    value: E,
    sink: &mut W,
    config: C,
) -> Result<usize, EncodeError> {
    let mut encoder = EncoderImpl::new(IoWriter::new(sink), config);
    value.encode(&mut encoder)?;
    Ok(encoder.into_writer().bytes_written())
}

pub(crate) struct IoWriter<'a, W: Write> {
    writer: &'a mut W,
    bytes_written: usize,
}

impl<'a, W: Write> IoWriter<'a, W> {
    pub(crate) fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            bytes_written: 0,
        }
    }

    pub(crate) const fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl<W: Write> Writer for IoWriter<'_, W> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.writer
            .write_all(bytes)
            .map_err(|inner| EncodeError::Io {
                inner,
                index: self.bytes_written,
            })?;
        self.bytes_written += bytes.len();
        Ok(())
    }
}

impl Encode for &CStr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.to_bytes().encode(encoder)
    }
}

impl Encode for CString {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_bytes().encode(encoder)
    }
}

impl<Context> Decode<Context> for CString {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        CString::new(std::vec::Vec::<u8>::decode(decoder)?).map_err(|error| {
            DecodeError::CStringNulError {
                position: error.nul_position(),
            }
        })
    }
}
crate::impl_borrow_decode!(CString);

impl Encode for &Path {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.to_str()
            .ok_or(EncodeError::InvalidPathCharacters)?
            .encode(encoder)
    }
}

impl<'de> BorrowDecode<'de, ()> for &'de Path {
    fn borrow_decode<D: BorrowDecoder<'de, Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Path::new(<&str>::borrow_decode(decoder)?))
    }
}

impl Encode for PathBuf {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_path().encode(encoder)
    }
}

impl<Context> Decode<Context> for PathBuf {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(String::decode(decoder)?.into())
    }
}
crate::impl_borrow_decode!(PathBuf);

impl Encode for IpAddr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::V4(address) => {
                0u32.encode(encoder)?;
                address.encode(encoder)
            }
            Self::V6(address) => {
                1u32.encode(encoder)?;
                address.encode(encoder)
            }
        }
    }
}

impl<Context> Decode<Context> for IpAddr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Self::V4(Ipv4Addr::decode(decoder)?)),
            1 => Ok(Self::V6(Ipv6Addr::decode(decoder)?)),
            found => Err(crate::error::DecodeError::UnexpectedVariant {
                found,
                allowed: &crate::error::RANGE_0_1,
                type_name: core::any::type_name::<IpAddr>(),
            }),
        }
    }
}
crate::impl_borrow_decode!(IpAddr);

impl Encode for Ipv4Addr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.writer().write(&self.octets())
    }
}

impl<Context> Decode<Context> for Ipv4Addr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(4)?;
        let mut bytes = [0u8; 4];
        decoder.reader().read(&mut bytes)?;
        Ok(Self::from(bytes))
    }
}
crate::impl_borrow_decode!(Ipv4Addr);

impl Encode for Ipv6Addr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.writer().write(&self.octets())
    }
}

impl<Context> Decode<Context> for Ipv6Addr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(16)?;
        let mut bytes = [0u8; 16];
        decoder.reader().read(&mut bytes)?;
        Ok(Self::from(bytes))
    }
}
crate::impl_borrow_decode!(Ipv6Addr);

impl Encode for SocketAddr {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Self::V4(address) => {
                0u32.encode(encoder)?;
                address.encode(encoder)
            }
            Self::V6(address) => {
                1u32.encode(encoder)?;
                address.encode(encoder)
            }
        }
    }
}

impl<Context> Decode<Context> for SocketAddr {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Self::V4(SocketAddrV4::decode(decoder)?)),
            1 => Ok(Self::V6(SocketAddrV6::decode(decoder)?)),
            found => Err(crate::error::DecodeError::UnexpectedVariant {
                found,
                allowed: &crate::error::RANGE_0_1,
                type_name: core::any::type_name::<SocketAddr>(),
            }),
        }
    }
}
crate::impl_borrow_decode!(SocketAddr);

impl Encode for SocketAddrV4 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.ip().encode(encoder)?;
        self.port().encode(encoder)
    }
}

impl<Context> Decode<Context> for SocketAddrV4 {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::new(Ipv4Addr::decode(decoder)?, u16::decode(decoder)?))
    }
}
crate::impl_borrow_decode!(SocketAddrV4);

impl Encode for SocketAddrV6 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.ip().encode(encoder)?;
        self.port().encode(encoder)
    }
}

impl<Context> Decode<Context> for SocketAddrV6 {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::new(
            Ipv6Addr::decode(decoder)?,
            u16::decode(decoder)?,
            0,
            0,
        ))
    }
}
crate::impl_borrow_decode!(SocketAddrV6);

impl Encode for SystemTime {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let duration = self
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|inner| EncodeError::InvalidSystemTime {
                inner,
                time: std::boxed::Box::new(*self),
            })?;
        duration.encode(encoder)
    }
}

impl<Context> Decode<Context> for SystemTime {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let duration = Duration::decode(decoder)?;
        SystemTime::UNIX_EPOCH
            .checked_add(duration)
            .ok_or(DecodeError::InvalidSystemTime { duration })
    }
}
crate::impl_borrow_decode!(SystemTime);

impl<T: Encode> Encode for Mutex<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.lock()
            .map_err(|_| EncodeError::LockFailed {
                type_name: core::any::type_name::<Mutex<T>>(),
            })?
            .encode(encoder)
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Mutex<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Mutex::new(T::decode(decoder)?))
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Mutex<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Mutex::new(T::borrow_decode(decoder)?))
    }
}

impl<T: Encode> Encode for RwLock<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.read()
            .map_err(|_| EncodeError::LockFailed {
                type_name: core::any::type_name::<RwLock<T>>(),
            })?
            .encode(encoder)
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for RwLock<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(RwLock::new(T::decode(decoder)?))
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for RwLock<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(RwLock::new(T::borrow_decode(decoder)?))
    }
}

impl<K: Encode, V: Encode, S> Encode for HashMap<K, V, S> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        crate::enc::encode_len(encoder, self.len())?;
        for (key, value) in self {
            key.encode(encoder)?;
            value.encode(encoder)?;
        }
        Ok(())
    }
}

impl<Context, K, V, S> Decode<Context> for HashMap<K, V, S>
where
    K: Decode<Context> + Eq + Hash,
    V: Decode<Context>,
    S: BuildHasher + Default,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(length)?;
        let mut map = HashMap::with_capacity_and_hasher(length, S::default());
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());
            map.insert(K::decode(decoder)?, V::decode(decoder)?);
        }
        Ok(map)
    }
}

impl<'de, Context, K, V, S> BorrowDecode<'de, Context> for HashMap<K, V, S>
where
    K: BorrowDecode<'de, Context> + Eq + Hash,
    V: BorrowDecode<'de, Context>,
    S: BuildHasher + Default,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(length)?;
        let mut map = HashMap::with_capacity_and_hasher(length, S::default());
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());
            map.insert(K::borrow_decode(decoder)?, V::borrow_decode(decoder)?);
        }
        Ok(map)
    }
}

impl<T: Encode, S> Encode for HashSet<T, S> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        crate::enc::encode_len(encoder, self.len())?;
        for value in self {
            value.encode(encoder)?;
        }
        Ok(())
    }
}

impl<Context, T, S> Decode<Context> for HashSet<T, S>
where
    T: Decode<Context> + Eq + Hash,
    S: BuildHasher + Default,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(length)?;
        let mut set = HashSet::with_capacity_and_hasher(length, S::default());
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());
            set.insert(T::decode(decoder)?);
        }
        Ok(set)
    }
}

impl<'de, Context, T, S> BorrowDecode<'de, Context> for HashSet<T, S>
where
    T: BorrowDecode<'de, Context> + Eq + Hash,
    S: BuildHasher + Default,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(length)?;
        let mut set = HashSet::with_capacity_and_hasher(length, S::default());
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());
            set.insert(T::borrow_decode(decoder)?);
        }
        Ok(set)
    }
}
