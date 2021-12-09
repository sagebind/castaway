//! This module contains helper functions and types used by the public-facing
//! macros. Some are public so they can be accessed by the expanded macro code,
//! but are not meant to be used by users directly and do not have a stable API.

use core::{
    any::{type_name, TypeId},
    marker::PhantomData,
    mem,
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
            Ok(unsafe { mem::transmute_copy::<T, U>(&mem::ManuallyDrop::new(value)) })
        } else {
            Err(value)
        }
    }
}

impl<T: 'static> TryCastOwned<T> for CastToken<T> {}

/// Determine if two types are equal to each other.
#[inline(always)]
fn type_eq<T: 'static, U: 'static>() -> bool {
    // Reduce the chance of `TypeId` collisions causing a problem by also
    // verifying the layouts match and the type names match. Since `T` and `U`
    // are known at compile time the compiler should optimize away these extra
    // checks anyway.
    mem::size_of::<T>() == mem::size_of::<U>()
        && mem::align_of::<T>() == mem::align_of::<U>()
        && TypeId::of::<T>() == TypeId::of::<U>()
        && type_name::<T>() == type_name::<U>()
}
