//! A `TypeId` comparison that does not require `'static` types.
//!
//! This is used to detect `u8` in generic positions so byte containers can be
//! copied in bulk. The implementation uses the standard `PhantomData` +
//! `dyn Any` technique; it is valid for any two types that differ only in
//! lifetimes, so it must only be used where the distinction is irrelevant.

use core::{any::TypeId, marker::PhantomData, mem};

/// Returns `true` when `A` and `B` are the same type, ignoring lifetimes.
pub(crate) fn same<A: ?Sized, B: ?Sized>() -> bool {
    non_static_type_id::<A>() == non_static_type_id::<B>()
}

fn non_static_type_id<T: ?Sized>() -> TypeId {
    trait NonStaticAny {
        fn type_id(&self) -> TypeId
        where
            Self: 'static;
    }

    impl<T: ?Sized> NonStaticAny for PhantomData<T> {
        fn type_id(&self) -> TypeId
        where
            Self: 'static,
        {
            TypeId::of::<T>()
        }
    }

    let marker = PhantomData::<T>;
    // SAFETY: `NonStaticAny::type_id` only calls `TypeId::of::<T>()`, which
    // requires `T: 'static`. Extending the trait object's lifetime does not
    // change which type is queried; the `'static` bound on the method is what
    // makes the call legal, and `PhantomData<T>` carries no data to drop.
    let object: &(dyn NonStaticAny + 'static) =
        unsafe { mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&marker) };
    object.type_id()
}

#[cfg(test)]
mod tests {
    #[test]
    fn distinguishes_types() {
        assert!(super::same::<u8, u8>());
        assert!(!super::same::<u8, i8>());
        assert!(!super::same::<u8, u16>());
        assert!(super::same::<&str, &str>());
    }
}
