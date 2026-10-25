//! Error types for encoding and decoding.

use core::fmt;

/// An error that occurred while encoding a value.
#[non_exhaustive]
#[derive(Debug)]
pub enum EncodeError {
    /// The destination ran out of space.
    UnexpectedEnd,

    /// A `RefCell` was already mutably borrowed.
    RefCellAlreadyBorrowed {
        /// The underlying borrow error.
        inner: core::cell::BorrowError,
        /// The type of the `RefCell` that was borrowed.
        type_name: &'static str,
    },

    /// A custom error with a static message.
    Other(&'static str),

    /// A custom error with an owned message.
    #[cfg(feature = "alloc")]
    OtherString(alloc::string::String),

    /// A path could not be represented as UTF-8.
    #[cfg(feature = "std")]
    InvalidPathCharacters,

    /// Writing to an I/O sink failed.
    #[cfg(feature = "std")]
    Io {
        /// The underlying I/O error.
        inner: std::io::Error,
        /// The number of bytes written before the failure.
        index: usize,
    },

    /// A `Mutex` or `RwLock` could not be acquired.
    #[cfg(feature = "std")]
    LockFailed {
        /// The type that could not be locked.
        type_name: &'static str,
    },

    /// A `SystemTime` predates the Unix epoch.
    #[cfg(feature = "std")]
    InvalidSystemTime {
        /// The underlying error.
        inner: std::time::SystemTimeError,
        /// The offending time.
        time: std::boxed::Box<std::time::SystemTime>,
    },

    /// A serde-specific encoding error.
    #[cfg(feature = "serde")]
    Serde(crate::serde::EncodeError),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => f.write_str("the destination ran out of space"),
            Self::RefCellAlreadyBorrowed { type_name, .. } => {
                write!(f, "`{type_name}` is already mutably borrowed")
            }
            Self::Other(message) => f.write_str(message),
            #[cfg(feature = "alloc")]
            Self::OtherString(message) => f.write_str(message),
            #[cfg(feature = "std")]
            Self::InvalidPathCharacters => f.write_str("path contains invalid UTF-8"),
            #[cfg(feature = "std")]
            Self::Io { inner, index } => {
                write!(f, "I/O error after writing {index} bytes: {inner}")
            }
            #[cfg(feature = "std")]
            Self::LockFailed { type_name } => write!(f, "failed to lock `{type_name}`"),
            #[cfg(feature = "std")]
            Self::InvalidSystemTime { .. } => f.write_str("system time predates the Unix epoch"),
            #[cfg(feature = "serde")]
            Self::Serde(error) => write!(f, "serde error: {error:?}"),
        }
    }
}

impl core::error::Error for EncodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::RefCellAlreadyBorrowed { inner, .. } => Some(inner),
            #[cfg(feature = "std")]
            Self::Io { inner, .. } => Some(inner),
            #[cfg(feature = "std")]
            Self::InvalidSystemTime { inner, .. } => Some(inner),
            _ => None,
        }
    }
}

/// An error that occurred while decoding a value.
#[non_exhaustive]
#[derive(Debug)]
pub enum DecodeError {
    /// The input ended before the value was complete.
    UnexpectedEnd {
        /// An estimate of how many additional bytes are required.
        additional: usize,
    },

    /// The configured size limit was exceeded.
    LimitExceeded,

    /// The encoded integer does not fit the requested type.
    InvalidIntegerType {
        /// The requested integer type.
        expected: IntegerType,
        /// The encoded integer type.
        found: IntegerType,
    },

    /// A `NonZero*` type was decoded from zero.
    NonZeroTypeIsZero {
        /// The requested integer type.
        non_zero_type: IntegerType,
    },

    /// An enum discriminant is outside the set of allowed variants.
    UnexpectedVariant {
        /// The type being decoded.
        type_name: &'static str,
        /// The allowed variants.
        allowed: &'static AllowedEnumVariants,
        /// The encoded discriminant.
        found: u32,
    },

    /// A string was not valid UTF-8.
    Utf8 {
        /// The underlying UTF-8 error.
        inner: core::str::Utf8Error,
    },

    /// A `char` was not valid Unicode scalar value encoding.
    InvalidCharEncoding([u8; 4]),

    /// A `bool` was encoded as a value other than `0` or `1`.
    InvalidBooleanValue(u8),

    /// The encoded array length does not match the requested array length.
    ArrayLengthMismatch {
        /// The length required by the type.
        required: usize,
        /// The length found in the input.
        found: usize,
    },

    /// The decoded value does not fit in `usize` on this platform.
    OutsideUsizeRange(u64),

    /// An enum without variants was decoded.
    EmptyEnum {
        /// The type being decoded.
        type_name: &'static str,
    },

    /// A `Duration` overflowed.
    InvalidDuration {
        /// The number of seconds.
        secs: u64,
        /// The number of nanoseconds.
        nanos: u32,
    },

    /// A `SystemTime` overflowed.
    InvalidSystemTime {
        /// The duration that could not be added to the Unix epoch.
        duration: core::time::Duration,
    },

    /// A `CString` contained an interior NUL byte.
    #[cfg(feature = "std")]
    CStringNulError {
        /// The position of the NUL byte.
        position: usize,
    },

    /// Reading from an I/O source failed.
    #[cfg(feature = "std")]
    Io {
        /// The underlying I/O error.
        inner: std::io::Error,
        /// An estimate of how many additional bytes are required.
        additional: usize,
    },

    /// A custom error with a static message.
    Other(&'static str),

    /// A custom error with an owned message.
    #[cfg(feature = "alloc")]
    OtherString(alloc::string::String),

    /// A serde-specific decoding error.
    #[cfg(feature = "serde")]
    Serde(crate::serde::DecodeError),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { additional } => {
                write!(
                    f,
                    "unexpected end of input, {additional} more bytes required"
                )
            }
            Self::LimitExceeded => f.write_str("the size limit was exceeded"),
            Self::InvalidIntegerType { expected, found } => {
                write!(f, "expected {expected:?}, found {found:?}")
            }
            Self::NonZeroTypeIsZero { non_zero_type } => {
                write!(f, "{non_zero_type:?} cannot be zero")
            }
            Self::UnexpectedVariant {
                type_name,
                allowed,
                found,
            } => match allowed {
                AllowedEnumVariants::Range { min, max } => {
                    write!(
                        f,
                        "invalid variant {found} for `{type_name}`, expected {min}..={max}"
                    )
                }
                AllowedEnumVariants::Allowed(values) => {
                    write!(
                        f,
                        "invalid variant {found} for `{type_name}`, expected {values:?}"
                    )
                }
            },
            Self::Utf8 { inner } => write!(f, "invalid UTF-8: {inner}"),
            Self::InvalidCharEncoding(bytes) => {
                write!(f, "invalid char encoding: {bytes:?}")
            }
            Self::InvalidBooleanValue(value) => {
                write!(f, "invalid boolean value: {value}")
            }
            Self::ArrayLengthMismatch { required, found } => {
                write!(f, "expected an array of length {required}, found {found}")
            }
            Self::OutsideUsizeRange(value) => {
                write!(f, "{value} does not fit in usize on this platform")
            }
            Self::EmptyEnum { type_name } => {
                write!(f, "cannot decode empty enum `{type_name}`")
            }
            Self::InvalidDuration { secs, nanos } => {
                write!(f, "duration overflows: {secs}s {nanos}ns")
            }
            Self::InvalidSystemTime { duration } => {
                write!(f, "system time overflows: {duration:?}")
            }
            #[cfg(feature = "std")]
            Self::CStringNulError { position } => {
                write!(f, "interior NUL byte at position {position}")
            }
            #[cfg(feature = "std")]
            Self::Io { inner, additional } => {
                write!(f, "I/O error ({additional} bytes requested): {inner}")
            }
            Self::Other(message) => f.write_str(message),
            #[cfg(feature = "alloc")]
            Self::OtherString(message) => f.write_str(message),
            #[cfg(feature = "serde")]
            Self::Serde(error) => write!(f, "serde error: {error:?}"),
        }
    }
}

impl core::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Utf8 { inner } => Some(inner),
            #[cfg(feature = "std")]
            Self::Io { inner, .. } => Some(inner),
            _ => None,
        }
    }
}

impl DecodeError {
    pub(crate) fn into_signed(self) -> Self {
        match self {
            Self::InvalidIntegerType { expected, found } => Self::InvalidIntegerType {
                expected: expected.into_signed(),
                found: found.into_signed(),
            },
            other => other,
        }
    }
}

pub(crate) const RANGE_0_1: AllowedEnumVariants = AllowedEnumVariants::Range { min: 0, max: 1 };
pub(crate) const RANGE_0_2: AllowedEnumVariants = AllowedEnumVariants::Range { min: 0, max: 2 };

/// The set of enum variants accepted by a decoder.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum AllowedEnumVariants {
    /// Any variant in the inclusive range is accepted.
    Range {
        /// The smallest accepted variant.
        min: u32,
        /// The largest accepted variant.
        max: u32,
    },
    /// Only these variants are accepted.
    Allowed(&'static [u32]),
}

/// An integer type, used for error reporting.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(missing_docs)]
pub enum IntegerType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    Reserved,
}

impl IntegerType {
    pub(crate) const fn into_signed(self) -> Self {
        match self {
            Self::U8 => Self::I8,
            Self::U16 => Self::I16,
            Self::U32 => Self::I32,
            Self::U64 => Self::I64,
            Self::U128 => Self::I128,
            Self::Usize => Self::Isize,
            other => other,
        }
    }
}
