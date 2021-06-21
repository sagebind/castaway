//! This module contains helper functions used by the public-facing macros. They
//! are public so they can be accessed by the expanded macro code, but are not
//! meant to be used by users directly and do not have a stable API.

use core::mem;

/// Attempt to cast a generic reference to a given type if the types are equal.
///
/// The reference does not have to be static as long as the reference target
/// type is static.
#[inline(always)]
pub fn try_cast_ref<'a: 'a, T: 'static, U: 'static>(value: &'a T) -> Result<&'a U, &'a T> {
    if type_eq::<T, U>() {
        Ok(unsafe { &*(value as *const T as *const U) })
    } else {
        Err(value)
    }
}

/// Attempt to cast a generic mutable reference to a given type if the types are
/// equal.
///
/// The reference does not have to be static as long as the reference target
/// type is static.
#[inline(always)]
pub fn try_cast_mut<'a: 'a, T: 'static, U: 'static>(value: &'a mut T) -> Result<&'a mut U, &'a mut T> {
    if type_eq::<T, U>() {
        Ok(unsafe { &mut *(value as *mut T as *mut U) })
    } else {
        Err(value)
    }
}

/// Attempt to cast a value to a given type if the types are equal.
#[inline(always)]
pub fn try_cast_owned<T: 'static, U: 'static>(value: T) -> Result<U, T> {
    if type_eq::<T, U>() {
        Ok(unsafe { mem::transmute_copy::<T, U>(&mem::ManuallyDrop::new(value)) })
    } else {
        Err(value)
    }
}

/// Determine if two types are equal to each other.
///
/// This implementation attempts to avoid the collision problems with `TypeId`
/// by relying on function monomorphization to distinguish between types with
/// additional memory layout sanity checks.
#[inline(always)]
fn type_eq<T: 'static, U: 'static>() -> bool {
    fn type_id_of<T>() -> usize {
        type_id_of::<T> as usize
    }

    mem::size_of::<T>() == mem::size_of::<U>()
        && mem::align_of::<T>() == mem::align_of::<U>()
        && type_id_of::<T>() == type_id_of::<U>()
}
