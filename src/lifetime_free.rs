/// Marker trait for types that do not contain any lifetime parameters. Such
/// types are safe to cast from non-static type parameters if their types are
/// equal.
///
/// This trait is used by [`cast!`] to determine what casts are legal on values
/// without a `'static` type constraint.
///
/// # Safety
///
/// When implementing this trait for a type, you must ensure that the type is
/// free of any lifetime parameters. Failure to meet **all** of the requirements
/// below may result in undefined behavior.
///
/// - The type must be `'static`.
/// - The type must be free of lifetime parameters. In other words, the type
///   must be an "owned" type and not contain *any* lifetime parameters.
/// - All contained fields must also be `LifetimeFree`.
///
/// # Examples
///
/// ```
/// use castaway::LifetimeFree;
///
/// struct Container<T>(T);
///
/// // UNDEFINED BEHAVIOR!!
/// // unsafe impl LifetimeFree for Container<&'static str> {}
///
/// // UNDEFINED BEHAVIOR!!
/// // unsafe impl<T> LifetimeFree for Container<T> {}
///
/// // This is safe.
/// unsafe impl<T: LifetimeFree> LifetimeFree for Container<T> {}
///
/// struct PlainOldData {
///     foo: u8,
///     bar: bool,
/// }
///
/// // This is also safe, since all fields are known to be `LifetimeFree`.
/// unsafe impl LifetimeFree for PlainOldData {}
/// ```
pub unsafe trait LifetimeFree {}

unsafe impl LifetimeFree for () {}
unsafe impl LifetimeFree for bool {}
unsafe impl LifetimeFree for char {}
unsafe impl LifetimeFree for f32 {}
unsafe impl LifetimeFree for f64 {}
unsafe impl LifetimeFree for i8 {}
unsafe impl LifetimeFree for i16 {}
unsafe impl LifetimeFree for i32 {}
unsafe impl LifetimeFree for i64 {}
unsafe impl LifetimeFree for i128 {}
unsafe impl LifetimeFree for isize {}
unsafe impl LifetimeFree for str {}
unsafe impl LifetimeFree for u8 {}
unsafe impl LifetimeFree for u16 {}
unsafe impl LifetimeFree for u32 {}
unsafe impl LifetimeFree for u64 {}
unsafe impl LifetimeFree for u128 {}
unsafe impl LifetimeFree for usize {}

unsafe impl LifetimeFree for core::num::NonZeroI8 {}
unsafe impl LifetimeFree for core::num::NonZeroI16 {}
unsafe impl LifetimeFree for core::num::NonZeroI32 {}
unsafe impl LifetimeFree for core::num::NonZeroI64 {}
unsafe impl LifetimeFree for core::num::NonZeroI128 {}
unsafe impl LifetimeFree for core::num::NonZeroIsize {}
unsafe impl LifetimeFree for core::num::NonZeroU8 {}
unsafe impl LifetimeFree for core::num::NonZeroU16 {}
unsafe impl LifetimeFree for core::num::NonZeroU32 {}
unsafe impl LifetimeFree for core::num::NonZeroU64 {}
unsafe impl LifetimeFree for core::num::NonZeroU128 {}

unsafe impl<T: LifetimeFree> LifetimeFree for [T] {}
#[rustversion::since(1.51)]
unsafe impl<T: LifetimeFree, const SIZE: usize> LifetimeFree for [T; SIZE] {}
unsafe impl<T: LifetimeFree> LifetimeFree for Option<T> {}
unsafe impl<T: LifetimeFree, E: LifetimeFree> LifetimeFree for Result<T, E> {}
unsafe impl<T: LifetimeFree> LifetimeFree for core::num::Wrapping<T> {}
unsafe impl<T: LifetimeFree> LifetimeFree for core::cell::Cell<T> {}
unsafe impl<T: LifetimeFree> LifetimeFree for core::cell::RefCell<T> {}

#[cfg(feature = "std")]
mod std_impls {
    use super::LifetimeFree;

    unsafe impl LifetimeFree for String {}

    unsafe impl<T: LifetimeFree> LifetimeFree for Box<T> {}
    unsafe impl<T: LifetimeFree> LifetimeFree for Vec<T> {}
    unsafe impl<T: LifetimeFree> LifetimeFree for std::sync::Arc<T> {}
}

/// A simple wrapper around a type to assert that it is lifetime-free.
///
/// When using [`cast!`], it is possible to cast generic type parameters to
/// their concrete type without `'static` bounds if the target concrete type
/// implements [`LifetimeFree`]. This trait is implemented for a number of types
/// in the standard library and can be implemented by your own types. However,
/// it is not possible for you to implement it on third-party types yourself, so
/// if you need to cast values to third-party types that ought to be marked as
/// lifetime-free, you can use this struct to do so for the purposes of a single
/// cast.
///
/// # Examples
///
/// ```
/// use castaway::{AssertLifetimeFree, cast};
///
/// // Try to downcast a generic type to `(u8, u8)` in a non-`'static` context. We
/// // know that it is lifetime-free despite not implementing `LifetimeFree`. This
/// // requires `unsafe`.
///
/// fn downcast_u8_pair<T>(value: T) -> Option<(u8, u8)> {
///     Some(cast!(unsafe { AssertLifetimeFree::new(value) }, AssertLifetimeFree<(u8, u8)>).ok()?.into_inner())
/// }
///
/// assert_eq!(downcast_u8_pair((0u8, 1u8)), Some((0u8, 1u8)));
/// assert_eq!(downcast_u8_pair((0u8, 1u16)), None);
/// ```
#[repr(transparent)]
pub struct AssertLifetimeFree<T>(T);

impl<T> AssertLifetimeFree<T> {
    pub const unsafe fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: LifetimeFree> From<T> for AssertLifetimeFree<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

unsafe impl<T> LifetimeFree for AssertLifetimeFree<T> {}
