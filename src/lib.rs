//! Experimental crate for zero-cost downcasting for limited runtime
//! specialization.
//!
//! This crate works fully on stable Rust, and also does not require the
//! standard library.

#![no_std]

use core::{any::TypeId, mem};

/// A trait for zero-cost type casting in generic contexts to allow limited
/// forms of specialization at runtime.
///
/// Similar to [`std::any::Any`], but does not require trait objects nor heap
/// allocation. Because of this, in most situations transmogrification will be
/// completely optimized away by the compiler, giving you effectively the same
/// performance as actual specialization.
///
/// # Examples
///
/// Specializing in a blanket trait implementation:
///
/// ```
/// use std::fmt::Display;
/// use transmogrify::Transmogrify;
///
/// /// Like `std::string::ToString`, but with an optimization when `Self` is
/// /// already a `String`.
/// ///
/// /// Since the standard library is allowed to use unstable features,
/// /// `ToString` already has this optimization using the `specialization`
/// /// feature, but this isn't something normal crates can do.
/// pub trait FastToString {
///     fn fast_to_string(&self) -> String;
/// }
///
/// // Currently transmogrify only works for static types...
/// impl<T: Display + 'static> FastToString for T {
///     fn fast_to_string(&self) -> String {
///         // If `T` is already a string, then take a different code path.
///         // After monomorphization, this check will be completely optimized
///         // away.
///         if let Some(string) = self.transmogrify_ref::<String>() {
///             // Don't invoke the std::fmt machinery, just clone the string.
///             string.to_owned()
///         } else {
///             // Make use of `Display` for any other `T`.
///             format!("{}", self)
///         }
///     }
/// }
///
/// println!("specialized: {}", String::from("hello").fast_to_string());
/// println!("default: {}", "hello".fast_to_string());
/// ```
pub trait Transmogrify: 'static {
    /// Get a reference to self if it is of type `T`, or `None` if it isn't.
    #[inline]
    fn transmogrify_ref<T>(&self) -> Option<&T>
    where
        T: Transmogrify,
    {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Some(unsafe { Self::transmogrify_ref_unchecked::<T>(self) })
        } else {
            None
        }
    }

    /// Get a mutable reference to self if it is of type `T`, or `None` if it
    /// isn't.
    #[inline]
    fn transmogrify_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Transmogrify,
    {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Some(unsafe { Self::transmogrify_mut_unchecked::<T>(self) })
        } else {
            None
        }
    }

    /// Convert self into a `T` if self is of type `T`, consuming self in the
    /// process.
    ///
    /// If self is not a `T`, returns self unchanged in an `Err`.
    #[inline]
    fn transmogrify_into<T>(self) -> Result<T, Self>
    where
        Self: Sized,
        T: Transmogrify + Sized,
    {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Ok(unsafe { Self::transmogrify_into_unchecked::<T>(self) })
        } else {
            Err(self)
        }
    }

    /// Cast a reference of self to type `T`. You can use this if you are
    /// already confident you know the type of `Self` and like to live
    /// dangerously.
    ///
    /// This is unsafe because self might not be a `T` and no checks are
    /// performed to prove otherwise.
    #[inline]
    unsafe fn transmogrify_ref_unchecked<T>(&self) -> &T
    where
        T: Transmogrify,
    {
        &*(self as *const Self as *const _)
    }

    /// Cast a mutable reference of self to type `T`. You can use this if you
    /// are already confident you know the type of `Self` and like to live
    /// dangerously.
    ///
    /// This is unsafe because self might not be a `T` and no checks are
    /// performed to prove otherwise.
    #[inline]
    unsafe fn transmogrify_mut_unchecked<T>(&mut self) -> &mut T
    where
        T: Transmogrify,
    {
        &mut *(self as *mut Self as *mut _)
    }

    /// Cast self to type `T`, consuming self and moving it. You can use this if
    /// you are already confident you know the type of `Self` and like to live
    /// dangerously.
    ///
    /// This is unsafe because self might not be a `T` and no checks are
    /// performed to prove otherwise.
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
    fn transmogrify_ref() {
        assert_eq!(42i32.transmogrify_ref::<i32>(), Some(&42i32));
        assert_eq!(42i32.transmogrify_ref::<u32>(), None);
    }

    #[test]
    fn transmogrify_mut() {
        assert_eq!(42i32.transmogrify_mut::<i32>(), Some(&mut 42i32));
        assert_eq!(42i32.transmogrify_mut::<u32>(), None);
    }

    #[test]
    fn transmogrify_into() {
        assert_eq!(42i32.transmogrify_into::<i32>(), Ok(42i32));
        assert_eq!(42i32.transmogrify_into::<u32>(), Err(42i32));
        assert_eq!((&42i32).transmogrify_into::<i32>(), Err(&42i32));
    }
}
