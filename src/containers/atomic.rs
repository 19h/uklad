use crate::{
    de::{Decode, Decoder},
    enc::{Encode, Encoder},
    error::{DecodeError, EncodeError},
};
use core::sync::atomic::Ordering;

macro_rules! impl_atomic {
    ($(#[$meta:meta])* $ty:ty, $inner:ty) => {
        $(#[$meta])*
        impl Encode for $ty {
            fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
                self.load(Ordering::SeqCst).encode(encoder)
            }
        }

        $(#[$meta])*
        impl<Context> Decode<Context> for $ty {
            fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
                Ok(<$ty>::new(<$inner>::decode(decoder)?))
            }
        }

        $(#[$meta])*
        crate::impl_borrow_decode!($ty);
    };
}

impl_atomic!(
    #[cfg(target_has_atomic = "8")]
    core::sync::atomic::AtomicBool,
    bool
);
impl_atomic!(
    #[cfg(target_has_atomic = "8")]
    core::sync::atomic::AtomicU8,
    u8
);
impl_atomic!(
    #[cfg(target_has_atomic = "16")]
    core::sync::atomic::AtomicU16,
    u16
);
impl_atomic!(
    #[cfg(target_has_atomic = "32")]
    core::sync::atomic::AtomicU32,
    u32
);
impl_atomic!(
    #[cfg(target_has_atomic = "64")]
    core::sync::atomic::AtomicU64,
    u64
);
impl_atomic!(
    #[cfg(target_has_atomic = "ptr")]
    core::sync::atomic::AtomicUsize,
    usize
);
impl_atomic!(
    #[cfg(target_has_atomic = "8")]
    core::sync::atomic::AtomicI8,
    i8
);
impl_atomic!(
    #[cfg(target_has_atomic = "16")]
    core::sync::atomic::AtomicI16,
    i16
);
impl_atomic!(
    #[cfg(target_has_atomic = "32")]
    core::sync::atomic::AtomicI32,
    i32
);
impl_atomic!(
    #[cfg(target_has_atomic = "64")]
    core::sync::atomic::AtomicI64,
    i64
);
impl_atomic!(
    #[cfg(target_has_atomic = "ptr")]
    core::sync::atomic::AtomicIsize,
    isize
);
