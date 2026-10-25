#![cfg(all(feature = "std", feature = "derive"))]

extern crate std;

use uklad::{Decode, Encode};
use std::borrow::Cow;
use std::string::String;

#[derive(Decode, Encode, PartialEq, Debug)]
#[uklad(
    decode_context = "()",
    borrow_decode_bounds = "&'__de U<'a, A>: ::uklad::de::BorrowDecode<'__de, ()> + '__de, '__de: 'a"
)]
struct T<'a, A: Clone + Encode + Decode<()>> {
    t: Cow<'a, U<'a, A>>,
}

#[derive(Clone, Decode, Encode, PartialEq, Debug)]
#[uklad(
    decode_context = "()",
    borrow_decode_bounds = "&'__de A: ::uklad::de::BorrowDecode<'__de, ()> + '__de, '__de: 'a"
)]
struct U<'a, A: Clone + Encode + Decode<()>> {
    u: Cow<'a, A>,
}

#[test]
fn test() {
    let u = U {
        u: Cow::Owned(String::from("Hello world")),
    };
    let t = T {
        t: Cow::Borrowed(&u),
    };
    let vec = uklad::encode_to_vec(&t, uklad::config::standard()).unwrap();

    let (decoded, len): (T<String>, usize) =
        uklad::decode_from_slice(&vec, uklad::config::standard()).unwrap();

    assert_eq!(t, decoded);
    assert_eq!(len, 12);
}
