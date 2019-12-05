/// This trait is an implementation detail for `Transmogrify` that sets
/// trait-level bounds for the target type of a cast. Using a separate trait
/// here makes it easier to provide alternative implementations.
pub trait Cast<T: ?Sized> {
    fn cast_ref(&self) -> Option<&T>;

    fn cast_mut(&mut self) -> Option<&mut T>;

    fn cast_into(self) -> Result<T, Self>
    where
        Self: Sized,
        T: Sized;

    unsafe fn cast_ref_unchecked(&self) -> &T;

    unsafe fn cast_mut_unchecked(&mut self) -> &mut T;

    unsafe fn cast_into_unchecked(self) -> T
    where
        Self: Sized,
        T: Sized;
}

#[cfg(feature = "specialization")]
mod specialization;

#[cfg(not(feature = "specialization"))]
mod type_id;
