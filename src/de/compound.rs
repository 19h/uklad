use super::{BorrowDecode, BorrowDecoder, Decode, Decoder, read::Reader};
use crate::error::{AllowedEnumVariants, DecodeError};
use core::{
    cell::{Cell, RefCell},
    cmp::Reverse,
    marker::PhantomData,
    num::Wrapping,
    ops::{Bound, Range, RangeInclusive},
    time::Duration,
};

impl<Context> Decode<Context> for () {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(_: &mut D) -> Result<Self, DecodeError> {
        Ok(())
    }
}
crate::impl_borrow_decode!(());

impl<Context, T: ?Sized> Decode<Context> for PhantomData<T> {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(_: &mut D) -> Result<Self, DecodeError> {
        Ok(PhantomData)
    }
}
crate::impl_borrow_decode!(PhantomData<T>, T: ?Sized);

impl<Context, T: Decode<Context>> Decode<Context> for Wrapping<T> {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Wrapping(T::decode(decoder)?))
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Wrapping<T> {
    #[inline]
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Wrapping(T::borrow_decode(decoder)?))
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Reverse<T> {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Reverse(T::decode(decoder)?))
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Reverse<T> {
    #[inline]
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Reverse(T::borrow_decode(decoder)?))
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Option<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match super::decode_option_variant(decoder, core::any::type_name::<Option<T>>())? {
            Some(()) => Ok(Some(T::decode(decoder)?)),
            None => Ok(None),
        }
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Option<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match super::decode_option_variant(decoder, core::any::type_name::<Option<T>>())? {
            Some(()) => Ok(Some(T::borrow_decode(decoder)?)),
            None => Ok(None),
        }
    }
}

fn unexpected_variant(
    found: u32,
    allowed: &'static AllowedEnumVariants,
    type_name: &'static str,
) -> DecodeError {
    DecodeError::UnexpectedVariant {
        found,
        allowed,
        type_name,
    }
}

impl<Context, T: Decode<Context>, U: Decode<Context>> Decode<Context> for Result<T, U> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Ok(T::decode(decoder)?)),
            1 => Ok(Err(U::decode(decoder)?)),
            found => Err(unexpected_variant(
                found,
                &crate::error::RANGE_0_1,
                core::any::type_name::<Result<T, U>>(),
            )),
        }
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>, U: BorrowDecode<'de, Context>>
    BorrowDecode<'de, Context> for Result<T, U>
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Ok(T::borrow_decode(decoder)?)),
            1 => Ok(Err(U::borrow_decode(decoder)?)),
            found => Err(unexpected_variant(
                found,
                &crate::error::RANGE_0_1,
                core::any::type_name::<Result<T, U>>(),
            )),
        }
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Cell<T> {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Cell::new(T::decode(decoder)?))
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Cell<T> {
    #[inline]
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Cell::new(T::borrow_decode(decoder)?))
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for RefCell<T> {
    #[inline]
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::decode(decoder)?))
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for RefCell<T> {
    #[inline]
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(RefCell::new(T::borrow_decode(decoder)?))
    }
}

impl<Context> Decode<Context> for Duration {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        const NANOS_PER_SEC: u64 = 1_000_000_000;
        let secs = u64::decode(decoder)?;
        let nanos = u32::decode(decoder)?;
        if secs.checked_add(nanos as u64 / NANOS_PER_SEC).is_none() {
            return Err(DecodeError::InvalidDuration { secs, nanos });
        }
        Ok(Duration::new(secs, nanos))
    }
}
crate::impl_borrow_decode!(Duration);

impl<Context, T: Decode<Context>> Decode<Context> for Range<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(T::decode(decoder)?..T::decode(decoder)?)
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Range<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(T::borrow_decode(decoder)?..T::borrow_decode(decoder)?)
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for RangeInclusive<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(RangeInclusive::new(
            T::decode(decoder)?,
            T::decode(decoder)?,
        ))
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for RangeInclusive<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(RangeInclusive::new(
            T::borrow_decode(decoder)?,
            T::borrow_decode(decoder)?,
        ))
    }
}

impl<Context, T: Decode<Context>> Decode<Context> for Bound<T> {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Bound::Unbounded),
            1 => Ok(Bound::Included(T::decode(decoder)?)),
            2 => Ok(Bound::Excluded(T::decode(decoder)?)),
            found => Err(unexpected_variant(
                found,
                &crate::error::RANGE_0_2,
                core::any::type_name::<Bound<T>>(),
            )),
        }
    }
}

impl<'de, Context, T: BorrowDecode<'de, Context>> BorrowDecode<'de, Context> for Bound<T> {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match u32::decode(decoder)? {
            0 => Ok(Bound::Unbounded),
            1 => Ok(Bound::Included(T::borrow_decode(decoder)?)),
            2 => Ok(Bound::Excluded(T::borrow_decode(decoder)?)),
            found => Err(unexpected_variant(
                found,
                &crate::error::RANGE_0_2,
                core::any::type_name::<Bound<T>>(),
            )),
        }
    }
}

impl<Context, T, const N: usize> Decode<Context> for [T; N]
where
    T: Decode<Context>,
{
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(core::mem::size_of::<[T; N]>())?;

        if unty::type_equal::<T, u8>() {
            let mut bytes = [0u8; N];
            decoder.reader().read(&mut bytes)?;
            // SAFETY: `T` is `u8`.
            Ok(unsafe { (&bytes as *const [u8; N] as *const [T; N]).read() })
        } else {
            let mut values: [Option<T>; N] = core::array::from_fn(|_| None);
            for slot in &mut values {
                decoder.unclaim_bytes_read(core::mem::size_of::<T>());
                *slot = Some(T::decode(decoder)?);
            }
            Ok(values.map(|value| value.expect("all slots are initialized")))
        }
    }
}

impl<'de, Context, T, const N: usize> BorrowDecode<'de, Context> for [T; N]
where
    T: BorrowDecode<'de, Context>,
{
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        decoder.claim_bytes_read(core::mem::size_of::<[T; N]>())?;

        if unty::type_equal::<T, u8>() {
            let mut bytes = [0u8; N];
            decoder.reader().read(&mut bytes)?;
            // SAFETY: `T` is `u8`.
            Ok(unsafe { (&bytes as *const [u8; N] as *const [T; N]).read() })
        } else {
            let mut values: [Option<T>; N] = core::array::from_fn(|_| None);
            for slot in &mut values {
                decoder.unclaim_bytes_read(core::mem::size_of::<T>());
                *slot = Some(T::borrow_decode(decoder)?);
            }
            Ok(values.map(|value| value.expect("all slots are initialized")))
        }
    }
}
