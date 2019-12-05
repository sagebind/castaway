use super::Cast;

impl<T, U> Cast<T> for U {
    default fn cast_ref(&self) -> Option<&T> {
        None
    }

    default fn cast_mut(&mut self) -> Option<&mut T> {
        None
    }

    default fn cast_into(self) -> Result<T, Self>
    where
        Self: Sized,
        T: Sized,
    {
        Err(self)
    }

    default unsafe fn cast_ref_unchecked(&self) -> &T {
        panic!("invalid cast")
    }

    default unsafe fn cast_mut_unchecked(&mut self) -> &mut T {
        panic!("invalid cast")
    }

    default unsafe fn cast_into_unchecked(self) -> T
    where
        Self: Sized,
        T: Sized,
    {
        panic!("invalid cast")
    }
}

impl<T> Cast<T> for T {
    fn cast_ref(&self) -> Option<&T> {
        Some(self)
    }

    fn cast_mut(&mut self) -> Option<&mut T> {
        Some(self)
    }

    fn cast_into(self) -> Result<T, Self>
    where
        Self: Sized,
        T: Sized,
    {
        Ok(self)
    }

    unsafe fn cast_ref_unchecked(&self) -> &T {
        self
    }

    unsafe fn cast_mut_unchecked(&mut self) -> &mut T {
        self
    }

    unsafe fn cast_into_unchecked(self) -> T
    where
        Self: Sized,
        T: Sized,
    {
        self
    }
}
