//! Low-level utility functions.

use core::{any::TypeId, marker::PhantomData, mem, ptr};

/// Determine if two static, generic types are equal to each other.
#[rustversion::attr(since(1.71), const)]
pub(crate) fn type_eq<T: 'static, U: 'static>() -> bool {
    // Reduce the chance of `TypeId` collisions causing a problem by also
    // verifying the layouts match and the type names match. Since `T` and `U`
    // are known at compile time the compiler should optimize away these extra
    // checks anyway.
    mem::size_of::<T>() == mem::size_of::<U>()
        && mem::align_of::<T>() == mem::align_of::<U>()
        && mem::needs_drop::<T>() == mem::needs_drop::<U>()
        && type_id_eq(TypeId::of::<T>(), TypeId::of::<U>())
}

pub(crate) trait TypeEq<U> {
    const EQ: bool;
}

impl<T: 'static, U: 'static> TypeEq<U> for T {
    const EQ: bool = type_eq::<T, U>();
}

#[rustversion::since(1.71)]
const fn type_id_eq(lhs: TypeId, rhs: TypeId) -> bool {
    // To do this comparison as `const` we compare the raw bytes of the two
    // `TypeId`s to check if they are identical. In general this isn't a good way
    // of doing this since we're relying on the internal representation of
    // `TypeId`, which isn't very nice of us.
    //
    // However, as of writing, Rust has always used unsigned integers as the raw
    // value, so this implementation works. Even if Rust began using pointers to
    // an underlying value, this implementation would still work, unless it were
    // possible for different pointers to be returned for the same type, which
    // would yield false negatives.

    let lhs = ptr::addr_of!(lhs).cast::<u8>();
    let rhs = ptr::addr_of!(rhs).cast::<u8>();
    let mut i = 0;

    while i < mem::size_of::<TypeId>() {
        unsafe {
            if lhs.add(i).read_unaligned() != rhs.add(i).read_unaligned() {
                return false;
            }
        }

        i += 1;
    }

    true
}

#[rustversion::before(1.71)]
fn type_id_eq(lhs: TypeId, rhs: TypeId) -> bool {
    lhs == rhs
}

/// Determine if two generic types (which might not be static) are equal to each
/// other.
///
/// This function must be used with extreme discretion, as no lifetime checking
/// is done. Meaning, this function considers `Struct<'a>` to be equal to
/// `Struct<'b>`, even if either `'a` or `'b` outlives the other.
#[inline(always)]
pub(crate) fn type_eq_non_static<T: ?Sized, U: ?Sized>() -> bool {
    non_static_type_id::<T>() == non_static_type_id::<U>()
}

/// Produces type IDs that are compatible with `TypeId::of::<T>`, but without
/// `T: 'static` bound.
fn non_static_type_id<T: ?Sized>() -> TypeId {
    trait NonStaticAny {
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static;
    }

    impl<T: ?Sized> NonStaticAny for PhantomData<T> {
        fn get_type_id(&self) -> TypeId
        where
            Self: 'static,
        {
            TypeId::of::<T>()
        }
    }

    {
        let phantom_data = PhantomData::<T>;
        NonStaticAny::get_type_id(unsafe {
            mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&phantom_data)
        })
    }
}

/// Reinterprets the bits of a value of one type as another type.
///
/// Similar to [`std::mem::transmute`], except that it makes no compile-time
/// guarantees about the layout of `T` or `U`, and is therefore even **more**
/// dangerous than `transmute`. Extreme caution must be taken when using this
/// function; it is up to the caller to assert that `T` and `U` have the same
/// size and layout and that it is safe to do this conversion. Which it probably
/// isn't, unless `T` and `U` are identical.
///
/// # Safety
///
/// It is up to the caller to uphold the following invariants:
///
/// - `T` must have the same size as `U`
/// - `T` must have the same alignment as `U`
/// - `T` must be safe to transmute into `U`
#[rustversion::since(1.56)]
pub(crate) const unsafe fn transmute_unchecked<T, U>(value: T) -> U {
    use core::mem::ManuallyDrop;

    union Transmute<T, U> {
        from: ManuallyDrop<T>,
        to: ManuallyDrop<U>,
    }

    ManuallyDrop::into_inner(
        Transmute {
            from: ManuallyDrop::new(value),
        }
        .to,
    )
}

#[rustversion::before(1.56)]
#[inline(always)]
pub(crate) unsafe fn transmute_unchecked<T, U>(value: T) -> U {
    let dest = ptr::read(&value as *const T as *const U);
    mem::forget(value);
    dest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_static_type_comparisons() {
        assert!(type_eq_non_static::<u8, u8>());
        assert!(type_eq_non_static::<&'static u8, &'static u8>());
        assert!(type_eq_non_static::<&u8, &'static u8>());

        assert!(!type_eq_non_static::<u8, i8>());
        assert!(!type_eq_non_static::<u8, &'static u8>());
    }
}
