//! Encoding and decoding configuration.
//!
//! The same configuration must be used for encoding and decoding.
//!
//! ```
//! let config = uklad::config::standard()
//!     .with_big_endian()
//!     .with_variable_int_encoding();
//! ```

use core::marker::PhantomData;

/// A complete encoder/decoder configuration.
///
/// The type parameters select the endianness, integer encoding, and size limit.
/// Use [`standard`], [`legacy`], or [`Configuration::default`] as a starting
/// point and adjust it with the `with_*` methods.
#[derive(Debug)]
pub struct Configuration<E = LittleEndian, I = Varint, L = NoLimit> {
    endianness: PhantomData<E>,
    int_encoding: PhantomData<I>,
    limit: PhantomData<L>,
}

impl<E, I, L> Copy for Configuration<E, I, L> {}

impl<E, I, L> Clone for Configuration<E, I, L> {
    fn clone(&self) -> Self {
        *self
    }
}

/// The default configuration: little endian with variable-length integers.
pub const fn standard() -> Configuration {
    generate()
}

/// Little endian with fixed-length integers.
pub const fn legacy() -> Configuration<LittleEndian, Fixint, NoLimit> {
    generate()
}

const fn generate<E, I, L>() -> Configuration<E, I, L> {
    Configuration {
        endianness: PhantomData,
        int_encoding: PhantomData,
        limit: PhantomData,
    }
}

impl<E, I, L> Default for Configuration<E, I, L> {
    fn default() -> Self {
        generate()
    }
}

impl<E, I, L> Configuration<E, I, L> {
    /// Encodes integers in big endian.
    pub const fn with_big_endian(self) -> Configuration<BigEndian, I, L> {
        generate()
    }

    /// Encodes integers in little endian.
    pub const fn with_little_endian(self) -> Configuration<LittleEndian, I, L> {
        generate()
    }

    /// Encodes integers using a variable-length representation.
    ///
    /// Unsigned values are encoded as follows:
    ///
    /// - `0..=250` as a single byte.
    /// - `251..=u16::MAX` as [`U16_MARKER`] followed by a `u16`.
    /// - `u16::MAX + 1..=u32::MAX` as [`U32_MARKER`] followed by a `u32`.
    /// - `u32::MAX + 1..=u64::MAX` as [`U64_MARKER`] followed by a `u64`.
    /// - `u64::MAX + 1..=u128::MAX` as [`U128_MARKER`] followed by a `u128`.
    ///
    /// Signed values use zigzag encoding before the unsigned representation.
    pub const fn with_variable_int_encoding(self) -> Configuration<E, Varint, L> {
        generate()
    }

    /// Encodes integers using a fixed-length representation.
    ///
    /// Enum discriminants are encoded as `u32`, and lengths as `u64`.
    pub const fn with_fixed_int_encoding(self) -> Configuration<E, Fixint, L> {
        generate()
    }

    /// Rejects inputs larger than `N` bytes.
    pub const fn with_limit<const N: usize>(self) -> Configuration<E, I, Limit<N>> {
        generate()
    }

    /// Removes the size limit.
    pub const fn with_no_limit(self) -> Configuration<E, I, NoLimit> {
        generate()
    }
}

/// Byte order used for multi-byte integers.
#[derive(PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Endianness {
    /// Little endian.
    Little,
    /// Big endian.
    Big,
}

/// Integer representation.
#[derive(PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum IntEncoding {
    /// Fixed-width integers.
    Fixed,
    /// Variable-width integers.
    Variable,
}

/// The configuration interface used by encoders and decoders.
///
/// This trait is sealed: it is implemented only for [`Configuration`].
pub trait Config: Copy + Clone + private::Sealed {
    /// The configured size limit, if any.
    const LIMIT: Option<usize>;
    /// The configured byte order.
    const ENDIANNESS: Endianness;
    /// The configured integer representation.
    const INT_ENCODING: IntEncoding;

    /// Returns the configured byte order.
    fn endianness(&self) -> Endianness {
        Self::ENDIANNESS
    }

    /// Returns the configured integer representation.
    fn int_encoding(&self) -> IntEncoding {
        Self::INT_ENCODING
    }

    /// Returns the configured size limit, if any.
    fn limit(&self) -> Option<usize> {
        Self::LIMIT
    }
}

/// Big endian byte order.
#[derive(Copy, Clone, Debug)]
pub struct BigEndian;

/// Little endian byte order.
#[derive(Copy, Clone, Debug)]
pub struct LittleEndian;

/// Fixed-width integer representation.
#[derive(Copy, Clone, Debug)]
pub struct Fixint;

/// Variable-width integer representation.
#[derive(Copy, Clone, Debug)]
pub struct Varint;

/// No size limit.
#[derive(Copy, Clone, Debug)]
pub struct NoLimit;

/// A size limit of `N` bytes.
#[derive(Copy, Clone, Debug)]
pub struct Limit<const N: usize>;

/// Marker byte introducing a 16-bit variable integer.
pub const U16_MARKER: u8 = 251;
/// Marker byte introducing a 32-bit variable integer.
pub const U32_MARKER: u8 = 252;
/// Marker byte introducing a 64-bit variable integer.
pub const U64_MARKER: u8 = 253;
/// Marker byte introducing a 128-bit variable integer.
pub const U128_MARKER: u8 = 254;

mod private {
    pub trait Sealed {}

    impl<E, I, L> Sealed for super::Configuration<E, I, L> {}
}

impl<E: ByteOrder, I: IntegerEncoding, L: SizeLimit> Config for Configuration<E, I, L> {
    const LIMIT: Option<usize> = L::VALUE;
    const ENDIANNESS: Endianness = E::VALUE;
    const INT_ENCODING: IntEncoding = I::VALUE;
}

impl<E: ByteOrder, I, L> ByteOrder for Configuration<E, I, L> {
    const VALUE: Endianness = E::VALUE;
}

impl<E, I: IntegerEncoding, L> IntegerEncoding for Configuration<E, I, L> {
    const VALUE: IntEncoding = I::VALUE;
}

impl<E, I, L: SizeLimit> SizeLimit for Configuration<E, I, L> {
    const VALUE: Option<usize> = L::VALUE;
}

pub(crate) trait ByteOrder {
    const VALUE: Endianness;
}

pub(crate) trait IntegerEncoding {
    const VALUE: IntEncoding;
}

pub(crate) trait SizeLimit {
    const VALUE: Option<usize>;
}

impl ByteOrder for BigEndian {
    const VALUE: Endianness = Endianness::Big;
}

impl ByteOrder for LittleEndian {
    const VALUE: Endianness = Endianness::Little;
}

impl IntegerEncoding for Fixint {
    const VALUE: IntEncoding = IntEncoding::Fixed;
}

impl IntegerEncoding for Varint {
    const VALUE: IntEncoding = IntEncoding::Variable;
}

impl SizeLimit for NoLimit {
    const VALUE: Option<usize> = None;
}

impl<const N: usize> SizeLimit for Limit<N> {
    const VALUE: Option<usize> = Some(N);
}
