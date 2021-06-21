//! Experimental crate for zero-cost downcasting for limited compile-time
//! specialization.
//!
//! This crate works fully on stable Rust, and also does not require the
//! standard library.
//!
//! Transmogrify provides the following key macros:
//!
//! - [`cast`]: Attempt to cast the result of an expression into a given
//!   concrete type.
//! - [`match_type`]

#![no_std]

#[doc(hidden)]
pub mod internal;

/// Attempt to cast the result of an expression into a given concrete type. If
/// the expression is in fact of the given type, an [`Ok`] is returned
/// containing the result of the expression as that type. If the types do not
/// match, the value is returned in an [`Err`] unchanged.
///
/// This macro is designed to work inside a generic context, and allows you to
/// downcast generic types to their concrete types or to another generic type at
/// compile time. If you are looking for the ability to downcast values at
/// runtime, you should use [`Any`](core::any::Any) instead.
///
/// Invoking this macro is zero-cost, meaning after normal compiler optimization
/// steps there will be no code generated for performing a cast. In debug builds
/// some glue code may be present with a small runtime cost.
///
/// # Restrictions
///
/// Attempting to perform an illegal or unsupported cast that can never be
/// successful, such as casting to a value with a longer lifetime than the
/// expression, will produce a compile-time error.
///
/// Due to language limitations with lifetime bounds, this macro is more
/// restrictive than what is theoretically possible and rejects some legal
/// casts. This is to ensure safety and correctness around lifetime handling.
/// Examples include the following:
///
/// - Casting an expression by value with a non-`'static` lifetime is not
///   allowed. For example, you cannot attempt to cast a `T: 'a` to `Foo<'a>`.
/// - Casting to a reference with a non-`'static` lifetime is not allowed if the
///   expression type is not required to be a reference. For example, you can
///   attempt to cast a `&T` to `&String`, but you can't attempt to cast a `T`
///   to `&String` because `T` may or may not be a reference. You can, however,
///   attempt to cast a `T: 'static` to `&'static String`.
/// - You cannot cast references whose target itself may contain non-`'static`
///   references. For example, you can attempt to cast a `&'a T: 'static` to
///   `&'a Foo<'static>`, but you can't attempt to cast a `&'a T: 'b` to `&'a
///   Foo<'b>`.
///
/// # Examples
///
/// Performing a trivial cast:
///
/// ```
/// use transmogrify::cast;
///
/// let value: u8 = 0;
/// assert_eq!(cast!(value, u8), Ok(0));
/// ```
///
/// Performing a cast in a generic context:
///
/// ```
/// use transmogrify::cast;
///
/// fn is_this_a_u8<T: 'static>(value: T) -> bool {
///     cast!(value, u8).is_ok()
/// }
///
/// assert!(is_this_a_u8(0u8));
/// assert!(!is_this_a_u8(0u16));
/// ```
///
/// Specialization in a blanket trait implementation:
///
/// ```
/// use std::fmt::Display;
/// use transmogrify::cast;
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
///     fn fast_to_string<'local>(&'local self) -> String {
///         // If `T` is already a string, then take a different code path.
///         // After monomorphization, this check will be completely optimized
///         // away.
///         //
///         // Note we can cast a `&'local self` to a `&'local String` as long
///         // as both `Self` and `String` are `'static`.
///         if let Ok(string) = cast!(self, &String) {
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
#[macro_export]
macro_rules! cast {
    // Cast to a mutable reference with a static lifetime. Since we know that
    // the target lifetime is static, we can allow casting values that aren't
    // statically known to be references, as long as the value is static.
    ($value:expr, &'static mut $T:path) => {{
        $crate::internal::try_cast_owned::<_, &'static mut $T>($value)
    }};

    // Cast a mutable reference with a named lifetime.
    ($value:expr, &$lifetime:lifetime mut $T:path) => {{
        $crate::internal::try_cast_mut::<$lifetime, _, $T>($value)
    }};

    // Cast a mutable reference with an inferred lifetime.
    ($value:expr, &mut $T:path) => {{
        $crate::internal::try_cast_mut::<_, $T>($value)
    }};

    // Cast to a reference with a static lifetime. Since we know that the target
    // lifetime is static, we can allow casting values that aren't statically
    // known to be references, as long as the value is static.
    ($value:expr, &'static $T:path) => {{
        $crate::internal::try_cast_owned::<_, &'static $T>($value)
    }};

    // Cast a reference with a named lifetime.
    ($value:expr, &$lifetime:lifetime $T:path) => {{
        $crate::internal::try_cast_ref::<$lifetime, _, $T>($value)
    }};

    // Cast a reference with an inferred lifetime.
    ($value:expr, &$T:path) => {{
        $crate::internal::try_cast_ref::<_, $T>($value)
    }};

    // Default case where we don't know if a reference is involved at macro
    // expansion time. Any cast can be made here, but both the value and the
    // cast target must be `'static`.
    ($value:expr, $T:ty) => {{
        $crate::internal::try_cast_owned::<_, $T>($value)
    }};
}

/// Construct a `match`-like expression that matches on a value's type, allowing
/// you to write conditional code depending on the compile-time type of an
/// expression.
///
/// This macro has all the same rules and restrictions as [`cast`].
///
/// # Examples
///
/// ```
/// use std::fmt::Display;
/// use transmogrify::match_type;
///
/// fn to_string<T: Display + 'static>(value: T) -> String {
///     match_type!(value, {
///         String as s => s,
///         &str as s => s.to_string(),
///         s => s.to_string(),
///     })
/// }
///
/// println!("{}", to_string("foo"));
/// ```
#[macro_export]
macro_rules! match_type {
    ($value:expr, {
        $T:ty as $pat:pat => $branch:expr,
        $default_pat:pat => $default_branch:expr $(,)*
    }) => {{
        match $crate::cast!($value, $T) {
            Ok($pat) => $branch,
            Err($default_pat) => $default_branch,
        }
    }};

    ($value:expr, {
        $T:ty as $pat:pat => $branch:expr,
        $($tail:tt)*
    }) => {{
        $crate::match_type!($value, {
            $T as $pat => $branch,
            value => $crate::match_type!(value, {
                $($tail)*
            })
        })
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast() {
        assert_eq!(cast!(0u8, u16), Err(0u8));
        assert_eq!(cast!(1u8, u8), Ok(1u8));
        assert_eq!(cast!(2u8, &'static u8), Err(2u8));

        static VALUE: u8 = 2u8;
        assert_eq!(cast!(&VALUE, &u8), Ok(&2u8));
        assert_eq!(cast!(&VALUE, &'static u8), Ok(&2u8));
        assert_eq!(cast!(&VALUE, &u16), Err(&2u8));
        assert_eq!(cast!(&VALUE, &i8), Err(&2u8));

        let value = 2u8;

        fn inner<'a>(value: &'a u8) {
            assert_eq!(cast!(value, &u8), Ok(&2u8));
            assert_eq!(cast!(value, &'a u8), Ok(&2u8));
            assert_eq!(cast!(value, &u16), Err(&2u8));
            assert_eq!(cast!(value, &i8), Err(&2u8));
        }

        inner(&value);
    }

    #[test]
    fn match_type() {
        let v = 42i32;

        assert!(match_type!(v, {
            u32 as _ => false,
            i32 as _ => true,
            _ => false,
        }));
    }
}
