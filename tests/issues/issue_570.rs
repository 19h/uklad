#![cfg(feature = "derive")]

#[derive(uklad::Encode, uklad::Decode)]
pub struct Eg<D, E> {
    data: (D, E),
}
