use super::{BorrowDecode, BorrowDecoder, Decode, Decoder};
use crate::error::DecodeError;

macro_rules! impl_tuple {
    ($($name:ident),+ $(,)?) => {
        impl<Context, $($name: Decode<Context>),+> Decode<Context> for ($($name,)+) {
            fn decode<DE: Decoder<Context = Context>>(decoder: &mut DE) -> Result<Self, DecodeError> {
                Ok(($($name::decode(decoder)?,)+))
            }
        }

        impl<'de, Context, $($name: BorrowDecode<'de, Context>),+> BorrowDecode<'de, Context>
            for ($($name,)+)
        {
            fn borrow_decode<DE: BorrowDecoder<'de, Context = Context>>(
                decoder: &mut DE,
            ) -> Result<Self, DecodeError> {
                Ok(($($name::borrow_decode(decoder)?,)+))
            }
        }
    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);
impl_tuple!(A, B, C, D, E, F, G, H, I);
impl_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
