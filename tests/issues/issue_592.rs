#![cfg(all(feature = "derive", feature = "std"))]

use uklad::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum TypeOfFile {
    Unknown = -1,
}
