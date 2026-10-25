use crate::{
    config::Config,
    de::{BorrowDecode, BorrowDecoder, Decode, Decoder, read::Reader},
    enc::{self, Encode, Encoder, write::SizeWriter, write::Writer},
    error::{DecodeError, EncodeError},
};
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    rc::Rc,
    string::String,
    vec::Vec,
};

#[cfg(target_has_atomic = "ptr")]
use alloc::sync::Arc;

/// A writer that appends to a `Vec<u8>`.
#[derive(Default)]
pub(crate) struct VecWriter {
    inner: Vec<u8>,
}

impl VecWriter {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn collect(self) -> Vec<u8> {
        self.inner
    }
}

impl Writer for VecWriter {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.inner.extend_from_slice(bytes);
        Ok(())
    }
}

/// Encodes `value` into a new `Vec<u8>`.
pub fn encode_to_vec<E: Encode, C: Config>(value: E, config: C) -> Result<Vec<u8>, EncodeError> {
    let capacity = {
        let mut encoder = enc::EncoderImpl::<_, C>::new(SizeWriter::default(), config);
        value.encode(&mut encoder)?;
        encoder.into_writer().bytes_written
    };

    let mut encoder = enc::EncoderImpl::<_, C>::new(VecWriter::with_capacity(capacity), config);
    value.encode(&mut encoder)?;
    Ok(encoder.into_writer().inner)
}

fn decode_sequence<T, D, F>(decoder: &mut D, mut decode: F) -> Result<Vec<T>, DecodeError>
where
    D: Decoder,
    F: FnMut(&mut D) -> Result<T, DecodeError>,
{
    let length = crate::de::decode_slice_len(decoder)?;
    decoder.claim_container_read::<T>(length)?;
    let mut values = Vec::with_capacity(length);
    for _ in 0..length {
        decoder.unclaim_bytes_read(core::mem::size_of::<T>());
        values.push(decode(decoder)?);
    }
    Ok(values)
}

impl<T: Encode> Encode for Vec<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        enc::encode_len(encoder, self.len())?;
        if unty::type_equal::<T, u8>() {
            // SAFETY: `T` is `u8`.
            let bytes = unsafe { core::slice::from_raw_parts(self.as_ptr().cast(), self.len()) };
            return encoder.writer().write(bytes);
        }
        for value in self {
            value.encode(encoder)?;
        }
        Ok(())
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Vec<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(length)?;

        if unty::type_equal::<T, u8>() {
            let mut bytes = alloc::vec![0u8; length];
            decoder.reader().read(&mut bytes)?;
            // SAFETY: `T` is `u8`.
            return Ok(unsafe { core::mem::transmute::<Vec<u8>, Vec<T>>(bytes) });
        }

        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());
            values.push(T::decode(decoder)?);
        }
        Ok(values)
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Vec<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<T>(length)?;

        if unty::type_equal::<T, u8>() {
            let mut bytes = alloc::vec![0u8; length];
            decoder.reader().read(&mut bytes)?;
            // SAFETY: `T` is `u8`.
            return Ok(unsafe { core::mem::transmute::<Vec<u8>, Vec<T>>(bytes) });
        }

        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());
            values.push(T::borrow_decode(decoder)?);
        }
        Ok(values)
    }
}

macro_rules! impl_sequence_encode {
    ($ty:ty $(, $bound:ident)?) => {
        impl<T: Encode $(+ $bound)?> Encode for $ty {
            fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
                enc::encode_len(encoder, self.len())?;
                for value in self {
                    value.encode(encoder)?;
                }
                Ok(())
            }
        }
    };
}

impl_sequence_encode!(VecDeque<T>);
impl_sequence_encode!(BinaryHeap<T>, Ord);
impl_sequence_encode!(BTreeSet<T>, Ord);

impl<Context, T: Decode<Context>> Decode<Context> for VecDeque<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(decode_sequence(decoder, T::decode)?.into())
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for VecDeque<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(decode_sequence(decoder, T::borrow_decode)?.into())
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for BinaryHeap<T>
where
    T: Ord,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(decode_sequence(decoder, T::decode)?.into())
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for BinaryHeap<T>
where
    T: Ord,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(decode_sequence(decoder, T::borrow_decode)?.into())
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for BTreeSet<T>
where
    T: Ord,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(decode_sequence(decoder, T::decode)?.into_iter().collect())
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for BTreeSet<T>
where
    T: Ord,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(decode_sequence(decoder, T::borrow_decode)?
            .into_iter()
            .collect())
    }
}

macro_rules! impl_map {
    ($ty:ty) => {
        impl<K: Encode, V: Encode> Encode for $ty {
            fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
                enc::encode_len(encoder, self.len())?;
                for (key, value) in self {
                    key.encode(encoder)?;
                    value.encode(encoder)?;
                }
                Ok(())
            }
        }
    };
}

impl_map!(BTreeMap<K, V>);

impl<Context, K: Decode<Context>, V: Decode<Context>> Decode<Context> for BTreeMap<K, V>
where
    K: Ord,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(length)?;
        let mut map = BTreeMap::new();
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());
            map.insert(K::decode(decoder)?, V::decode(decoder)?);
        }
        Ok(map)
    }
}

impl<'de, Context, K: BorrowDecode<'de, Context>, V: BorrowDecode<'de, Context>>
    BorrowDecode<'de, Context> for BTreeMap<K, V>
where
    K: Ord,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let length = crate::de::decode_slice_len(decoder)?;
        decoder.claim_container_read::<(K, V)>(length)?;
        let mut map = BTreeMap::new();
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<(K, V)>());
            map.insert(K::borrow_decode(decoder)?, V::borrow_decode(decoder)?);
        }
        Ok(map)
    }
}

impl Encode for String {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_bytes().encode(encoder)
    }
}

impl<Context> Decode<Context> for String {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        String::from_utf8(Vec::<u8>::decode(decoder)?).map_err(|error| DecodeError::Utf8 {
            inner: error.utf8_error(),
        })
    }
}
crate::impl_borrow_decode!(String);

impl<Context> Decode<Context> for Box<str> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(String::decode(decoder)?.into_boxed_str())
    }
}
crate::impl_borrow_decode!(Box<str>);

impl<T: Encode + ?Sized> Encode for Box<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        T::encode(self, encoder)
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Box<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Box::new(T::decode(decoder)?))
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Box<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Box::new(T::borrow_decode(decoder)?))
    }
}

impl<Context, T: Decode<Context> + 'static> Decode<Context> for Box<[T]> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::decode(decoder)?.into_boxed_slice())
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context> + 'de> BorrowDecode<'de, Context> for Box<[T]> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::borrow_decode(decoder)?.into_boxed_slice())
    }
}

impl<T: Encode + ?Sized> Encode for Rc<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        T::encode(self, encoder)
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Rc<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Rc::new(T::decode(decoder)?))
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Rc<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Rc::new(T::borrow_decode(decoder)?))
    }
}

impl<Context> Decode<Context> for Rc<str> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(String::decode(decoder)?.into())
    }
}
crate::impl_borrow_decode!(Rc<str>);

impl<Context, T: Decode<Context> + 'static> Decode<Context> for Rc<[T]> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::decode(decoder)?.into())
    }
}
impl<'de, Context, T: BorrowDecode<'de, Context> + 'de> BorrowDecode<'de, Context> for Rc<[T]> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::borrow_decode(decoder)?.into())
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<T: Encode + ?Sized> Encode for Arc<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        T::encode(self, encoder)
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<Context, T: Decode<Context>> Decode<Context> for Arc<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Arc::new(T::decode(decoder)?))
    }
}
#[cfg(target_has_atomic = "ptr")]
impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Arc<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Arc::new(T::borrow_decode(decoder)?))
    }
}

#[cfg(target_has_atomic = "ptr")]
impl<Context> Decode<Context> for Arc<str> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(String::decode(decoder)?.into())
    }
}
#[cfg(target_has_atomic = "ptr")]
crate::impl_borrow_decode!(Arc<str>);

#[cfg(target_has_atomic = "ptr")]
impl<Context, T: Decode<Context> + 'static> Decode<Context> for Arc<[T]> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::decode(decoder)?.into())
    }
}
#[cfg(target_has_atomic = "ptr")]
impl<'de, Context, T: BorrowDecode<'de, Context> + 'de> BorrowDecode<'de, Context> for Arc<[T]> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Vec::<T>::borrow_decode(decoder)?.into())
    }
}

impl<T: Encode + ?Sized> Encode for Cow<'_, T>
where
    T: ToOwned,
    for<'a> &'a T: Encode,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_ref().encode(encoder)
    }
}

impl<Context, T> Decode<Context> for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Cow::Owned(T::Owned::decode(decoder)?))
    }
}

impl<'de, Context, T> BorrowDecode<'de, Context> for Cow<'de, T>
where
    T: ToOwned + ?Sized,
    &'de T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Cow::Borrowed(<&T>::borrow_decode(decoder)?))
    }
}
