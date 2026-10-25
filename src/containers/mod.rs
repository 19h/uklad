//! Container implementations selected by crate features.

#[cfg(feature = "alloc")]
pub(crate) mod alloc;
#[cfg(feature = "alloc")]
pub use self::alloc::*;

#[cfg(feature = "std")]
pub(crate) mod std;
#[cfg(feature = "std")]
pub use self::std::*;

mod atomic;
