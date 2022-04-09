//! This module contains helper functions and types used by the public-facing
//! macros. Some are public so they can be accessed by the expanded macro code,
//! but are not meant to be used by users directly and do not have a stable API.

use core::{
    any::{type_name, TypeId},
    marker::PhantomData,
    mem,
    ptr,
};

/// A token struct used to capture the type of a value without taking ownership of
/// it. Used to select a cast implementation in macros.
pub struct CastToken<T>(PhantomData<T>);

impl<T> CastToken<T> {
    /// Create a cast token for the given type of value.
    pub fn of_val(_value: &T) -> Self {
        Self(PhantomData)
    }
}

/// Supporting trait for autoderef specialization on mutable slices.
pub trait TryCastSliceMut<'a, T: 'static> {
    /// Attempt to cast a generic mutable slice to a given type if the types are
    /// equal.
    ///
    /// The reference does not have to be static as long as the item type is
    /// static.
    #[inline(always)]
    fn try_cast<U: 'static>(&self, value: &'a mut [T]) -> Result<&'a mut [U], &'a mut [T]> {
        if type_eq::<T, U>() {
            Ok(unsafe { &mut *(value as *mut [T] as *mut [U]) })
        } else {
            Err(value)
        }
    }
}

impl<'a, T: 'static> TryCastSliceMut<'a, T> for &&&&CastToken<&'a mut [T]> {}

/// Supporting trait for autoderef specialization on slices.
pub trait TryCastSliceRef<'a, T: 'static> {
    /// Attempt to cast a generic slice to a given type if the types are equal.
    ///
    /// The reference does not have to be static as long as the item type is
    /// static.
    #[inline(always)]
    fn try_cast<U: 'static>(&self, value: &'a [T]) -> Result<&'a [U], &'a [T]> {
        if type_eq::<T, U>() {
            Ok(unsafe { &*(value as *const [T] as *const [U]) })
        } else {
            Err(value)
        }
    }
}

impl<'a, T: 'static> TryCastSliceRef<'a, T> for &&&CastToken<&'a [T]> {}

/// Supporting trait for autoderef specialization on mutable references.
pub trait TryCastMut<'a, T: 'static> {
    /// Attempt to cast a generic mutable reference to a given type if the types
    /// are equal.
    ///
    /// The reference does not have to be static as long as the reference target
    /// type is static.
    #[inline(always)]
    fn try_cast<U: 'static>(&self, value: &'a mut T) -> Result<&'a mut U, &'a mut T> {
        if type_eq::<T, U>() {
            Ok(unsafe { &mut *(value as *mut T as *mut U) })
        } else {
            Err(value)
        }
    }
}

impl<'a, T: 'static> TryCastMut<'a, T> for &&CastToken<&'a mut T> {}

/// Supporting trait for autoderef specialization on references.
pub trait TryCastRef<'a, T: 'static> {
    /// Attempt to cast a generic reference to a given type if the types are
    /// equal.
    ///
    /// The reference does not have to be static as long as the reference target
    /// type is static.
    #[inline(always)]
    fn try_cast<U: 'static>(&self, value: &'a T) -> Result<&'a U, &'a T> {
        if type_eq::<T, U>() {
            Ok(unsafe { &*(value as *const T as *const U) })
        } else {
            Err(value)
        }
    }
}

impl<'a, T: 'static> TryCastRef<'a, T> for &CastToken<&'a T> {}

/// Default trait for autoderef specialization.
pub trait TryCastOwned<T: 'static> {
    /// Attempt to cast a value to a given type if the types are equal.
    #[inline(always)]
    fn try_cast<U: 'static>(&self, value: T) -> Result<U, T> {
        if type_eq::<T, U>() {
            Ok(unsafe { transmute_owned::<T, U>(value) })
        } else {
            Err(value)
        }
    }
}

impl<T: 'static> TryCastOwned<T> for CastToken<T> {}

pub trait TryCastLifetimeFree<T> {
    #[inline(always)]
    fn try_cast<U: LifetimeFree>(&self, value: T) -> Result<U, T> {
        try_cast_lifetime_free(value)
    }
}

impl<T> TryCastLifetimeFree<T> for CastToken<T> {}

/// Marker trait for types that do not contain any lifetime parameters. Such types
/// are safe to cast from non-static sources if their types are equal.
pub unsafe trait LifetimeFree: 'static {}

unsafe impl LifetimeFree for bool {}
unsafe impl LifetimeFree for u8 {}
unsafe impl LifetimeFree for u16 {}
unsafe impl LifetimeFree for u32 {}
unsafe impl LifetimeFree for u64 {}
unsafe impl LifetimeFree for i8 {}
unsafe impl LifetimeFree for i16 {}
unsafe impl LifetimeFree for i32 {}
unsafe impl LifetimeFree for i64 {}

/// Attempt to cast a potentially non-static value to a given lifetime-free type
/// if the types are equal.
#[inline(always)]
pub fn try_cast_lifetime_free<T, U: LifetimeFree>(value: T) -> Result<U, T> {
    // SAFETY: If `U` is lifetime-free, and the base types of `T` and `U` are
    // equal, then `T` is also lifetime-free. Therefore `T` and `U` are strictly
    // identical and it is safe to cast a `T` into a `U`.
    //
    // We know that `U` is lifetime-free because of the `LifetimeFree` trait
    // checked statically. `LifetimeFree` is an unsafe trait implemented for
    // individual types, so the burden of verifying that a type is indeed
    // lifetime-free is on the implementer.

    if type_eq_non_static::<T, U>() {
        Ok(unsafe { transmute_owned::<T, U>(value) })
    } else {
        Err(value)
    }
}

/// Determine if two static, generic types are equal to each other.
#[inline(always)]
fn type_eq<T: 'static, U: 'static>() -> bool {
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
/// This function must be used with extreme discretion, as no lifetime checking is
/// done.
#[inline(always)]
fn type_eq_non_static<T, U>() -> bool {
    // Inline has a weird, but desirable result on this function. It can't be
    // fully inlined everywhere since it creates a function pointer of itself.
    // But in practice when used here, the act of taking the address will be
    // inlined, thus avoiding a function call when comparing two types.
    #[inline]
    fn type_id_of<T>() -> usize {
        type_id_of::<T> as usize
    }

    mem::size_of::<T>() == mem::size_of::<U>()
        && mem::align_of::<T>() == mem::align_of::<U>()
        && mem::needs_drop::<T>() == mem::needs_drop::<U>()
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
        && type_id_of::<T>() == type_id_of::<U>()
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
/// size and that it is safe to do this conversion. Which it probably isn't,
/// unless `T` and `U` are identical.
///
/// A `const` function on Rust 1.56 and newer.
///
/// # Safety
///
/// It is up to the caller to uphold the following invariants:
///
/// - `T` must have the same size as `U`
/// - `T` must have the same alignment as `U`
/// - `T` must be safe to transmute into `U`
#[inline(always)]
unsafe fn transmute_owned<T, U>(value: T) -> U {
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
        assert!(!type_eq_non_static::<u8, i8>());
    }
}
