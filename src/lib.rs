#![no_std]
#![warn(missing_docs, unused_lifetimes)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! A compact binary encoder and decoder.
//!
//! The format stores values without field names or other metadata, so encoded
//! data is typically as small as, or smaller than, the in-memory value.
//!
//! # Features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std`   | yes     | `std::io` readers and writers, plus standard containers. |
//! | `alloc` | yes     | `Vec`, `String`, `Box`, and other alloc types. |
//! | `derive`| yes     | `#[derive(Encode, Decode)]` macros. |
//! | `serde` | no      | serde interoperability. |
//!
//! Atomic integer types are supported on targets that provide them.
//!
//! # Choosing an API
//!
//! | Situation | Encode | Decode |
//! |-----------|--------|--------|
//! | Files and network streams | [`encode_into_std_write`] | [`decode_from_std_read`] |
//! | In-memory buffers | [`encode_to_vec`] | [`decode_from_slice`] |
//! | Custom reader or writer | [`encode_into_writer`] | [`decode_from_reader`] |
//! | Pre-allocated buffers | [`encode_into_slice`] | [`decode_from_slice`] |
//!
//! # Example
//!
//! ```
//! # #[cfg(all(feature = "derive", feature = "alloc"))]
//! # mod example {
//! use uklad::{Decode, Encode};
//!
//! #[derive(Encode, Decode, PartialEq, Debug)]
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! pub fn run() {
//!     let point = Point { x: 10, y: -4 };
//!     let bytes = uklad::encode_to_vec(&point, uklad::config::standard()).unwrap();
//!     let (decoded, read): (Point, usize) =
//!         uklad::decode_from_slice(&bytes, uklad::config::standard()).unwrap();
//!
//!     assert_eq!(decoded, point);
//!     assert_eq!(read, bytes.len());
//! }
//! # }
//! # #[cfg(all(feature = "derive", feature = "alloc"))]
//! # example::run();
//! ```
//!
//! [`encode_into_std_write`]: https://docs.rs/uklad/latest/uklad/fn.encode_into_std_write.html
//! [`decode_from_std_read`]: https://docs.rs/uklad/latest/uklad/fn.decode_from_std_read.html
//! [`encode_to_vec`]: https://docs.rs/uklad/latest/uklad/fn.encode_to_vec.html

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(any(feature = "std", test))]
extern crate std;

mod containers;
pub(crate) mod type_id;
pub(crate) mod varint;

#[cfg(any(
    feature = "alloc",
    feature = "std",
    feature = "derive",
    feature = "serde"
))]
pub use containers::*;

pub mod config;
pub mod de;
pub mod enc;
pub mod error;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde;

pub use de::{BorrowDecode, Decode};
pub use enc::Encode;

#[cfg(feature = "derive")]
pub use uklad_derive::{BorrowDecode, Decode, Encode};

use config::Config;
use de::read::Reader;
use enc::write::Writer;

/// Encodes `value` into `dst` and returns the number of bytes written.
pub fn encode_into_slice<E: Encode, C: Config>(
    value: E,
    dst: &mut [u8],
    config: C,
) -> Result<usize, error::EncodeError> {
    let mut encoder = enc::EncoderImpl::new(enc::write::SliceWriter::new(dst), config);
    value.encode(&mut encoder)?;
    Ok(encoder.into_writer().bytes_written())
}

/// Encodes `value` into a custom [`Writer`].
pub fn encode_into_writer<E: Encode, W: Writer, C: Config>(
    value: E,
    writer: W,
    config: C,
) -> Result<(), error::EncodeError> {
    let mut encoder = enc::EncoderImpl::new(writer, config);
    value.encode(&mut encoder)?;
    Ok(())
}

/// Decodes a value from `src` and returns it together with the number of bytes
/// read.
///
/// This does not support borrowed output types such as `&str` or `&[u8]`; use
/// [`borrow_decode_from_slice`] for those.
pub fn decode_from_slice<D: Decode<()>, C: Config>(
    src: &[u8],
    config: C,
) -> Result<(D, usize), error::DecodeError> {
    decode_from_slice_with_context(src, config, ())
}

/// Decodes a value from `src` with a context.
///
/// This does not support borrowed output types such as `&str` or `&[u8]`; use
/// [`borrow_decode_from_slice_with_context`] for those.
pub fn decode_from_slice_with_context<Context, D: Decode<Context>, C: Config>(
    src: &[u8],
    config: C,
    context: Context,
) -> Result<(D, usize), error::DecodeError> {
    let reader = de::read::SliceReader::new(src);
    let mut decoder = de::DecoderImpl::new(reader, config, context);
    let value = D::decode(&mut decoder)?;
    Ok((value, decoder.bytes_read()))
}

/// Decodes a value that may borrow from `src`.
pub fn borrow_decode_from_slice<'a, D: BorrowDecode<'a, ()>, C: Config>(
    src: &'a [u8],
    config: C,
) -> Result<(D, usize), error::DecodeError> {
    borrow_decode_from_slice_with_context(src, config, ())
}

/// Decodes a value that may borrow from `src` with a context.
pub fn borrow_decode_from_slice_with_context<
    'a,
    Context,
    D: BorrowDecode<'a, Context>,
    C: Config,
>(
    src: &'a [u8],
    config: C,
    context: Context,
) -> Result<(D, usize), error::DecodeError> {
    let reader = de::read::SliceReader::new(src);
    let mut decoder = de::DecoderImpl::new(reader, config, context);
    let value = D::borrow_decode(&mut decoder)?;
    Ok((value, decoder.bytes_read()))
}

/// Decodes a value from a custom [`Reader`].
pub fn decode_from_reader<D: Decode<()>, R: Reader, C: Config>(
    reader: R,
    config: C,
) -> Result<D, error::DecodeError> {
    let mut decoder = de::DecoderImpl::new(reader, config, ());
    D::decode(&mut decoder)
}

#[cfg(all(feature = "alloc", feature = "derive", doc))]
pub mod spec {
    #![doc = include_str!("../docs/spec.md")]
}

#[cfg(all(feature = "alloc", feature = "derive", doctest))]
mod readme {
    #![doc = include_str!("../README.md")]
}
