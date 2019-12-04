//! Experimental crate for zero-cost downcasting for limited runtime
//! specialization.

#![no_std]

use core::{any::TypeId, mem};

/// A trait for zero-cost type casting in generic contexts to allow limited
/// forms of specialization at runtime.
///
/// Similar to [`std::any::Any`], but does not require trait objects nor heap
/// allocation. Because of this, in most situations transmogrification will be
/// completely optimized away by the compiler, giving you effectively the same
/// performance as actual specialization.
pub trait Transmogrify: 'static {
    /// Get a reference to self if it is of type `T`, or `None` if it isn't.
    fn transmogrify_ref<T>(&self) -> Option<&T>
    where
        T: Transmogrify,
    {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Some(unsafe { self.transmogrify_ref_unchecked() })
        } else {
            None
        }
    }

    /// Get a mutable reference to self if it is of type `T`, or `None` if it
    /// isn't.
    fn transmogrify_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Transmogrify,
    {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Some(unsafe { self.transmogrify_mut_unchecked() })
        } else {
            None
        }
    }

    /// Convert self into a `T` if self is of type `T`, consuming self in the
    /// process.
    ///
    /// If self is not a `T`, returns self unchanged in an `Err`.
    fn transmogrify_into<T>(self) -> Result<T, Self>
    where
        Self: Sized,
        T: Transmogrify + Sized,
    {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Ok(unsafe { self.transmogrify_into_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Cast a reference of self to type `T`.
    ///
    /// This is unsafe because self might not be a `T`.
    #[inline]
    unsafe fn transmogrify_ref_unchecked<T>(&self) -> &T
    where
        T: Transmogrify,
    {
        &*(self as *const Self as *const _)
    }

    /// Cast a mutable reference of self to type `T`.
    ///
    /// This is unsafe because self might not be a `T`.
    #[inline]
    unsafe fn transmogrify_mut_unchecked<T>(&mut self) -> &mut T
    where
        T: Transmogrify,
    {
        &mut *(self as *mut Self as *mut _)
    }

    /// Cast self to type `T`, consuming self and moving it.
    ///
    /// This is unsafe because self might not be a `T`.
    #[inline]
    unsafe fn transmogrify_into_unchecked<T>(self) -> T
    where
        Self: Sized,
        T: Transmogrify + Sized,
    {
        let out = mem::transmute_copy(&self);
        mem::forget(self);
        out
    }
}

/// Any static type can be transmogrified.
impl<T: 'static> Transmogrify for T {}

#[cfg(test)]
mod tests {
    use super::Transmogrify;

    #[test]
    fn transmogrify_into() {
        assert!(0i32.transmogrify_into::<i32>().is_ok());
        assert!(0i32.transmogrify_into::<u32>().is_err());
        assert!((&0i32).transmogrify_into::<i32>().is_err());
    }
}
