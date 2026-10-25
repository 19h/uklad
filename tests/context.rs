#![cfg(all(feature = "alloc", feature = "derive"))]

use std::vec::Vec;

use uklad::{
    BorrowDecode, Decode, Encode, de::BorrowDecoder, decode_from_slice_with_context,
    error::DecodeError,
};

/// A minimal arena used as a decoding context.
struct Arena;

impl Arena {
    fn alloc<T>(&self, values: Vec<T>) -> &[T] {
        Box::leak(values.into_boxed_slice())
    }
}

#[derive(PartialEq, Eq, Debug)]
struct CodableVec<'arena, T>(&'arena [T]);

impl<T: Encode> Encode for CodableVec<'_, T> {
    fn encode<E: uklad::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), uklad::error::EncodeError> {
        self.0.encode(encoder)
    }
}

impl<'arena, T: Decode<&'arena Arena>> Decode<&'arena Arena> for CodableVec<'arena, T>
where
    T: Clone,
{
    fn decode<D: uklad::de::Decoder<Context = &'arena Arena>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let length = u64::decode(decoder)?;
        let length = usize::try_from(length).map_err(|_| DecodeError::OutsideUsizeRange(length))?;
        decoder.claim_container_read::<T>(length)?;

        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());
            values.push(T::decode(decoder)?);
        }

        let arena = *decoder.context();
        Ok(Self(arena.alloc(values)))
    }
}

impl<'de, 'arena, T: BorrowDecode<'de, &'arena Arena>> BorrowDecode<'de, &'arena Arena>
    for CodableVec<'arena, T>
where
    T: Clone,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = &'arena Arena>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let length = u64::decode(decoder)?;
        let length = usize::try_from(length).map_err(|_| DecodeError::OutsideUsizeRange(length))?;
        decoder.claim_container_read::<T>(length)?;

        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            decoder.unclaim_bytes_read(core::mem::size_of::<T>());
            values.push(T::borrow_decode(decoder)?);
        }

        let arena = *decoder.context();
        Ok(Self(arena.alloc(values)))
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Debug)]
#[uklad(decode_context = "&'bump Arena")]
struct Container<'bump> {
    vec: CodableVec<'bump, u32>,
}

#[derive(Encode, Decode, PartialEq, Eq, Debug)]
#[uklad(decode_context = "&'bump Arena")]
enum EnumContainer<'bump> {
    Vec(CodableVec<'bump, u32>),
}

#[test]
fn decode_with_context() {
    let config = uklad::config::standard();
    let arena = Arena;

    let container = Container {
        vec: CodableVec(arena.alloc(std::vec![1, 2, 3])),
    };

    let bytes = uklad::encode_to_vec(&container, config).unwrap();
    let (decoded, _) =
        decode_from_slice_with_context::<_, Container, _>(&bytes, config, &arena).unwrap();
    assert_eq!(container, decoded);

    let enum_container = EnumContainer::Vec(CodableVec(arena.alloc(std::vec![4, 5])));
    let bytes = uklad::encode_to_vec(&enum_container, config).unwrap();
    let (decoded, _) =
        decode_from_slice_with_context::<_, EnumContainer, _>(&bytes, config, &arena).unwrap();
    assert_eq!(enum_container, decoded);
}
