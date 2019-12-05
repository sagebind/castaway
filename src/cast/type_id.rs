use super::Cast;
use core::{any::TypeId, mem};

// Note that we can only cast from static types to other static types because of
// the requirement on `TypeId`.
impl<T: 'static, U: 'static> Cast<T> for U {
    #[inline]
    fn cast_ref(&self) -> Option<&T> {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Some(unsafe { Self::cast_ref_unchecked(self) })
        } else {
            None
        }
    }

    #[inline]
    fn cast_mut(&mut self) -> Option<&mut T> {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Some(unsafe { Self::cast_mut_unchecked(self) })
        } else {
            None
        }
    }

    #[inline]
    fn cast_into(self) -> Result<T, Self>
    where
        Self: Sized,
        T: Sized,
    {
        if TypeId::of::<Self>() == TypeId::of::<T>() {
            Ok(unsafe { Self::cast_into_unchecked(self) })
        } else {
            Err(self)
        }
    }

    #[inline]
    unsafe fn cast_ref_unchecked(&self) -> &T {
        &*(self as *const Self as *const _)
    }

    #[inline]
    unsafe fn cast_mut_unchecked(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut _)
    }

    #[inline]
    unsafe fn cast_into_unchecked(self) -> T
    where
        Self: Sized,
        T: Sized,
    {
        // On nightly we can use unions to view the same region of memory as
        // multiple types without copying.
        #[cfg(feature = "union-transmute")]
        {
            union Transmute<T, U> {
                src: mem::ManuallyDrop<T>,
                dest: mem::ManuallyDrop<U>,
            }

            mem::ManuallyDrop::into_inner(Transmute {
                src: mem::ManuallyDrop::new(self),
            }.dest)
        }

        // This involves a memory copy that should get optimized away, but is
        // nonetheless inferior to the nightly implementation.
        #[cfg(not(feature = "union-transmute"))]
        {
            let out = mem::transmute_copy(&self);
            mem::forget(self);
            out
        }
    }
}
