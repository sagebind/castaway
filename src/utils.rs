//! Low-level utility functions.

use core::{
    any::{type_name, TypeId},
    mem, ptr,
};

/// Determine if two static, generic types are equal to each other.
#[inline(always)]
pub(crate) fn type_eq<T: 'static, U: 'static>() -> bool {
    // Reduce the chance of `TypeId` collisions causing a problem by also
    // verifying the layouts match and the type names match. Since `T` and `U`
    // are known at compile time the compiler should optimize away these extra
    // checks anyway.
    mem::size_of::<T>() == mem::size_of::<U>()
        && mem::align_of::<T>() == mem::align_of::<U>()
        && mem::needs_drop::<T>() == mem::needs_drop::<U>()
        && TypeId::of::<T>() == TypeId::of::<U>()
        && type_name::<T>() == type_name::<U>()
}

/// Determine if two generic types which may not be static are equal to each
/// other.
///
/// This function must be used with extreme discretion, as no lifetime checking
/// is done. Meaning, this function considers `Struct<'a>` to be equal to
/// `Struct<'b>`, even if either `'a` or `'b` outlives the other.
#[inline(always)]
pub(crate) fn type_eq_non_static<T: ?Sized, U: ?Sized>() -> bool {
    // Inline has a weird, but desirable result on this function. It can't be
    // fully inlined everywhere since it creates a function pointer of itself.
    // But in practice when used here, the act of taking the address will be
    // inlined, thus avoiding a function call when comparing two types.
    #[inline]
    fn type_id_of<T: ?Sized>() -> usize {
        type_id_of::<T> as usize
    }

    // What we're doing here is comparing two function pointers of the same
    // generic function to see if they are identical. If they are not
    // identical then `T` and `U` are not the same type.
    //
    // If they are equal, then they _might_ be the same type, unless an
    // optimization step reduced two different functions to the same
    // implementation due to having the same body. To avoid this we are using
    // a function which references itself. This is something that LLVM cannot
    // merge, since each monomorphized function has a reference to a different
    // global alias.
    type_id_of::<T>() == type_id_of::<U>()
        // This is used as a sanity check more than anything. Our previous calls
        // should not have any false positives, but if they did then the odds of
        // them having the same type name as well is extremely unlikely.
        && type_name::<T>() == type_name::<U>()
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
