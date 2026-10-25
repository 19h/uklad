#![cfg(all(feature = "derive", feature = "std"))]

use uklad::{Decode, Encode};

#[derive(Encode, Decode)]
struct Foo<Bar = ()> {
    x: Bar,
}
