//! This module contains helper traits and types used by the public-facing
//! macros. Most are public so they can be accessed by the expanded macro code,
//! but are not meant to be used by users directly and do not have a stable API.
//!
//! The various `TryCast*` traits in this module are referenced in macro
//! expansions and expose multiple possible implementations of casting with
//! different generic bounds. The compiler chooses which trait to use to fulfill
//! the cast based on the trait bounds using the _autoderef_ trick.

use crate::{
    lifetime_free::LifetimeFree,
    utils::{transmute_unchecked, type_eq, type_eq_non_static},
};
use core::{marker::PhantomData, ops::Deref};

/// This type is the core of how we exercise the autoderef trick. Rather than
/// using traits implementing a method at different reference levels, the target
/// value is wrapped in multiple layers of this struct, which implements
/// [`Deref`] to yield the inner value.
///
/// This allows us to implement multiple method definitions using the same name at
/// different wrapping levels, since we own this type.
pub struct AutoDerefLayer<T: ?Sized>(pub T);

impl<T> Deref for AutoDerefLayer<AutoDerefLayer<T>> {
    type Target = AutoDerefLayer<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A token struct used to capture a type without taking ownership of any
/// values. Used to select a cast implementation in macros.
pub struct TypeToken<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> TypeToken<T> {
    /// Create a token for the given type of value.
    pub const fn of_val(_value: &T) -> Self {
        Self::of()
    }

    /// Create a new token of the specified type.
    pub const fn of() -> Self {
        Self(PhantomData)
    }
}

/// Supporting trait for autoderef specialization on mutable references to lifetime-free
/// types.
///
/// TryCastMutLifetimeFree
impl<'a, T, U: LifetimeFree>
    AutoDerefLayer<
        AutoDerefLayer<
            AutoDerefLayer<
                AutoDerefLayer<
                    AutoDerefLayer<
                        AutoDerefLayer<
                            AutoDerefLayer<(TypeToken<&'a mut T>, TypeToken<&'a mut U>)>,
                        >,
                    >,
                >,
            >,
        >,
    >
{
    #[inline(always)]
    pub fn try_cast(&self, value: &'a mut T) -> Result<&'a mut U, &'a mut T> {
        // SAFETY: See comments on safety in `TryCastLifetimeFree`.

        if type_eq_non_static::<T, U>() {
            // Pointer casts are not allowed here since the compiler can't prove
            // that `&mut T` and `&mut U` have the same kind of associated
            // pointer data if they are fat pointers. But we know they are
            // identical, so we use a transmute.
            Ok(unsafe { transmute_unchecked::<&mut T, &mut U>(value) })
        } else {
            Err(value)
        }
    }
}

/// Supporting trait for autoderef specialization on references to lifetime-free
/// types.
///
/// TryCastRefLifetimeFree
impl<'a, T, U: LifetimeFree>
    AutoDerefLayer<
        AutoDerefLayer<
            AutoDerefLayer<
                AutoDerefLayer<
                    AutoDerefLayer<AutoDerefLayer<(TypeToken<&'a T>, TypeToken<&'a U>)>>,
                >,
            >,
        >,
    >
{
    #[inline(always)]
    pub fn try_cast(&self, value: &'a T) -> Result<&'a U, &'a T> {
        // SAFETY: See comments on safety in `TryCastLifetimeFree`.

        if type_eq_non_static::<T, U>() {
            // Pointer casts are not allowed here since the compiler can't prove
            // that `&T` and `&U` have the same kind of associated pointer data if
            // they are fat pointers. But we know they are identical, so we use
            // a transmute.
            Ok(unsafe { transmute_unchecked::<&T, &U>(value) })
        } else {
            Err(value)
        }
    }
}

/// Supporting trait for autoderef specialization on lifetime-free types.
///
/// TryCastOwnedLifetimeFree
impl<'a, T, U: LifetimeFree>
    AutoDerefLayer<
        AutoDerefLayer<
            AutoDerefLayer<AutoDerefLayer<AutoDerefLayer<(TypeToken<T>, TypeToken<U>)>>>,
        >,
    >
{
    #[inline(always)]
    pub fn try_cast(&self, value: T) -> Result<U, T> {
        // SAFETY: If `U` is lifetime-free, and the base types of `T` and `U`
        // are equal, then `T` is also lifetime-free. Therefore `T` and `U` are
        // strictly identical and it is safe to cast a `T` into a `U`.
        //
        // We know that `U` is lifetime-free because of the `LifetimeFree` trait
        // checked statically. `LifetimeFree` is an unsafe trait implemented for
        // individual types, so the burden of verifying that a type is indeed
        // lifetime-free is on the implementer.

        if type_eq_non_static::<T, U>() {
            Ok(unsafe { transmute_unchecked::<T, U>(value) })
        } else {
            Err(value)
        }
    }
}

/// Supporting trait for autoderef specialization on mutable slices.
///
/// TryCastSliceMut
impl<'a, T: 'static, U: 'static>
    AutoDerefLayer<
        AutoDerefLayer<
            AutoDerefLayer<AutoDerefLayer<(TypeToken<&'a mut [T]>, TypeToken<&'a mut [U]>)>>,
        >,
    >
{
    /// Attempt to cast a generic mutable slice to a given type if the types are
    /// equal.
    ///
    /// The reference does not have to be static as long as the item type is
    /// static.
    #[inline(always)]
    pub fn try_cast(&self, value: &'a mut [T]) -> Result<&'a mut [U], &'a mut [T]> {
        std::dbg!();
        if type_eq::<T, U>() {
            Ok(unsafe { &mut *(value as *mut [T] as *mut [U]) })
        } else {
            Err(value)
        }
    }
}

/// Supporting trait for autoderef specialization on slices.
///
/// TryCastSliceRef
impl<'a, T: 'static, U: 'static>
    AutoDerefLayer<AutoDerefLayer<AutoDerefLayer<(TypeToken<&'a [T]>, TypeToken<&'a [U]>)>>>
{
    /// Attempt to cast a generic slice to a given type if the types are equal.
    ///
    /// The reference does not have to be static as long as the item type is
    /// static.
    #[inline(always)]
    pub fn try_cast(&self, value: &'a [T]) -> Result<&'a [U], &'a [T]> {
        if std::dbg!(type_eq::<T, U>()) {
            Ok(unsafe { &*(value as *const [T] as *const [U]) })
        } else {
            Err(value)
        }
    }
}

/// Supporting trait for autoderef specialization on mutable references.
///
/// TryCastMut
impl<'a, T: 'static, U: 'static>
    AutoDerefLayer<AutoDerefLayer<(TypeToken<&'a mut T>, TypeToken<&'a mut U>)>>
{
    /// Attempt to cast a generic mutable reference to a given type if the types
    /// are equal.
    ///
    /// The reference does not have to be static as long as the reference target
    /// type is static.
    #[inline(always)]
    pub fn try_cast(&self, value: &'a mut T) -> Result<&'a mut U, &'a mut T> {
        if type_eq::<T, U>() {
            Ok(unsafe { &mut *(value as *mut T as *mut U) })
        } else {
            Err(value)
        }
    }
}

impl<'a, T: 'static, U: 'static>
    AutoDerefLayer<AutoDerefLayer<(TypeToken<&'a T>, TypeToken<&'a U>)>>
{
    /// Attempt to cast a generic reference to a given type if the types are
    /// equal.
    ///
    /// The reference does not have to be static as long as the reference target
    /// type is static.
    #[inline(always)]
    pub fn try_cast(&self, value: &'a T) -> Result<&'a U, &'a T> {
        if type_eq::<T, U>() {
            Ok(unsafe { &*(value as *const T as *const U) })
        } else {
            Err(value)
        }
    }
}

impl<'a, T: 'static, U: 'static> AutoDerefLayer<(TypeToken<T>, TypeToken<U>)> {
    /// Attempt to cast a value to a given type if the types are equal.
    ///
    /// This is the default `try_cast` implementation.
    #[inline(always)]
    pub fn try_cast(&self, value: T) -> Result<U, T> {
        if type_eq::<T, U>() {
            Ok(unsafe { transmute_unchecked::<T, U>(value) })
        } else {
            Err(value)
        }
    }
}
