use super::{Encode, Encoder, encode_len, encode_option, write::Writer};
use crate::error::EncodeError;
use core::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    ops::{Bound, Range, RangeInclusive},
    time::Duration,
};

impl Encode for () {
    #[inline]
    fn encode<E: Encoder>(&self, _: &mut E) -> Result<(), EncodeError> {
        Ok(())
    }
}

impl<T: ?Sized> Encode for PhantomData<T> {
    #[inline]
    fn encode<E: Encoder>(&self, _: &mut E) -> Result<(), EncodeError> {
        Ok(())
    }
}

impl<T: Encode + ?Sized> Encode for &T {
    #[inline]
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        T::encode(self, encoder)
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encode_option(encoder, &self.as_ref().map(|_| ()))?;
        if let Some(value) = self {
            value.encode(encoder)?;
        }
        Ok(())
    }
}

impl<T: Encode, U: Encode> Encode for Result<T, U> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Ok(value) => {
                0u32.encode(encoder)?;
                value.encode(encoder)
            }
            Err(error) => {
                1u32.encode(encoder)?;
                error.encode(encoder)
            }
        }
    }
}

impl<T: Encode> Encode for Cell<T>
where
    T: Copy,
{
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.get().encode(encoder)
    }
}

impl<T: Encode + ?Sized> Encode for RefCell<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let guard = self
            .try_borrow()
            .map_err(|inner| EncodeError::RefCellAlreadyBorrowed {
                inner,
                type_name: core::any::type_name::<RefCell<T>>(),
            })?;
        T::encode(&guard, encoder)
    }
}

impl Encode for Duration {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_secs().encode(encoder)?;
        self.subsec_nanos().encode(encoder)
    }
}

impl<T: Encode> Encode for Range<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.start.encode(encoder)?;
        self.end.encode(encoder)
    }
}

impl<T: Encode> Encode for RangeInclusive<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.start().encode(encoder)?;
        self.end().encode(encoder)
    }
}

impl<T: Encode> Encode for Bound<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Bound::Unbounded => 0u32.encode(encoder),
            Bound::Included(value) => {
                1u32.encode(encoder)?;
                value.encode(encoder)
            }
            Bound::Excluded(value) => {
                2u32.encode(encoder)?;
                value.encode(encoder)
            }
        }
    }
}

impl<T: Encode> Encode for [T] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encode_len(encoder, self.len())?;

        if crate::type_id::same::<T, u8>() {
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

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        if crate::type_id::same::<T, u8>() {
            // SAFETY: `T` is `u8`.
            let bytes = unsafe { core::slice::from_raw_parts(self.as_ptr().cast(), N) };
            encoder.writer().write(bytes)
        } else {
            for value in self {
                value.encode(encoder)?;
            }
            Ok(())
        }
    }
}

impl Encode for str {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_bytes().encode(encoder)
    }
}
