use std::ops::Deref;
use std::ptr;

/// An item interned by an `Interner`.
///
/// As two equal interned items will always be the same reference,
/// only the reference is compared for equality rather than going
/// to the potentially expensive implementation on the concrete type.
///
/// In all other cases, this should act as if it were a reference to the wrapped type.
/// If you just want to get the wrapped reference out, see `Interned::get`.
#[derive(Debug, Ord, PartialOrd, Hash)]
pub struct Interned<'a, T: 'a + ?Sized>(pub(crate) &'a T);

impl<'a, T: 'a + ?Sized> Interned<'a, T> {
    /// Get the wrapped reference out directly.
    ///
    /// This is an associated fn rather than a method in order to avoid
    /// conflicting with implementations on the concrete type.
    pub fn get(this: &Self) -> &'a T {
        this.0
    }
}

impl<'a, T: 'a + ?Sized> Copy for Interned<'a, T> {}
impl<'a, T: 'a + ?Sized> Clone for Interned<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: 'a + ?Sized> Eq for Interned<'a, T> {}
impl<'a, T: 'a + ?Sized> PartialEq for Interned<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl<'a, T: 'a + ?Sized> Deref for Interned<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}
