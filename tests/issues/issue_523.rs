#![cfg(all(feature = "derive", feature = "std"))]

extern crate std;

use std::borrow::Cow;
use uklad::{Decode, Encode};

#[derive(Clone, Encode, Decode)]
pub struct Foo<'a>(Cow<'a, str>);
