use core::{borrow::Borrow, ops::Deref, ptr};

/// An item interned by an `Interner`.
///
/// As two equal interned items will always be the same reference, only the
/// reference is compared for equality rather than going to the potentially
/// expensive `PartialEq` implementation on the concrete type.
///
/// In all other cases, this should act exactly as if it were a reference to the
/// wrapped type. To get the wrapped reference with the full lifetime, see
/// [`Interned::get`].
#[derive(Debug, Ord, PartialOrd, Hash)]
pub struct Interned<'a, T: ?Sized>(pub(crate) &'a T);

impl<'a, T: ?Sized> Interned<'a, T> {
    /// Get the wrapped reference out directly.
    ///
    /// This is an associated function rather than a method in order to
    /// avoid conflicting with implementations on the concrete type.
    pub fn get(this: &Self) -> &'a T {
        this.0
    }
}

impl<T: ?Sized> Copy for Interned<'_, T> {}
impl<T: ?Sized> Clone for Interned<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Eq for Interned<'_, T> {}
impl<T: ?Sized> PartialEq for Interned<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl<T: ?Sized> Deref for Interned<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized> Borrow<T> for Interned<'_, T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T, U> AsRef<U> for Interned<'_, T>
where
    T: AsRef<U> + ?Sized,
    U: ?Sized,
{
    fn as_ref(&self) -> &U {
        (**self).as_ref()
    }
}
