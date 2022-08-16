use {
    crate::Interned,
    std::{
        borrow::Borrow,
        collections::hash_map::RandomState,
        fmt,
        hash::{BuildHasher, Hash, Hasher},
        marker::PhantomData,
        ops::Deref,
        ptr::NonNull,
    },
};

#[cfg(feature = "raw")]
use hashbrown::hash_map::RawEntryMut;
#[cfg(feature = "hashbrown")]
use hashbrown::hash_map::{Entry, HashMap};
#[cfg(not(feature = "hashbrown"))]
use std::collections::hash_map::{Entry, HashMap};

#[cfg(not(feature = "parking_lot"))]
use std::sync::RwLock;
#[cfg(feature = "parking_lot")]
use {crate::parking_lot_shim::*, parking_lot::RwLock};

/// A wrapper around box that does not provide &mut access to the pointee and
/// uses raw-pointer borrowing rules to avoid invalidating extant references.
///
/// The resolved reference is guaranteed valid until the PinBox is dropped.
struct PinBox<T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<Box<T>>,
}

impl<T: ?Sized> PinBox<T> {
    fn new(x: Box<T>) -> Self {
        Self {
            ptr: NonNull::new(Box::into_raw(x)).unwrap(),
            _marker: PhantomData,
        }
    }

    #[allow(unsafe_code)]
    unsafe fn as_ref<'a>(&self) -> &'a T {
        self.ptr.as_ref()
    }
}

impl<T: ?Sized> Drop for PinBox<T> {
    fn drop(&mut self) {
        #[allow(unsafe_code)] // SAFETY: PinBox acts like Box.
        unsafe {
            Box::from_raw(self.ptr.as_ptr())
        };
    }
}

impl<T: ?Sized> Deref for PinBox<T> {
    type Target = T;
    #[allow(unsafe_code)] // SAFETY: PinBox acts like Box.
    fn deref(&self) -> &T {
        unsafe { self.as_ref() }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for PinBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + Eq> Eq for PinBox<T> {}
impl<T: ?Sized + PartialEq> PartialEq for PinBox<T> {
    fn eq(&self, other: &Self) -> bool {
        (**self).eq(&**other)
    }
}
impl<T: ?Sized + PartialEq> PartialEq<T> for PinBox<T> {
    fn eq(&self, other: &T) -> bool {
        (**self).eq(other)
    }
}

impl<T: ?Sized + Ord> Ord for PinBox<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}
impl<T: ?Sized + PartialOrd> PartialOrd for PinBox<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}
impl<T: ?Sized + PartialOrd> PartialOrd<T> for PinBox<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

impl<T: ?Sized + Hash> Hash for PinBox<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl<T: ?Sized> Borrow<T> for PinBox<T> {
    fn borrow(&self) -> &T {
        self
    }
}

#[allow(unsafe_code)] // SAFETY: PinBox acts like Box.
unsafe impl<T: ?Sized> Send for PinBox<T> where Box<T>: Send {}

#[allow(unsafe_code)] // SAFETY: PinBox acts like Box.
unsafe impl<T: ?Sized> Sync for PinBox<T> where Box<T>: Sync {}

/// An interner based on a `HashSet`. See the crate-level docs for more.
#[derive(Debug)]
pub struct Interner<T: ?Sized, S = RandomState> {
    arena: RwLock<HashMap<PinBox<T>, (), S>>,
}

impl<T: ?Sized, S: Default> Default for Interner<T, S> {
    fn default() -> Self {
        Interner {
            arena: RwLock::default(),
        }
    }
}

impl<T: Eq + Hash + ?Sized, S: BuildHasher> Interner<T, S> {
    /// Intern an item into the interner.
    ///
    /// Takes borrowed or heap-allocated items. If the item has not been
    /// previously interned, it will be `Into::into`ed a `Box` on the heap and
    /// cached. Notably, if you give this fn a `String` or `Vec`, the allocation
    /// will be shrunk to fit.
    ///
    /// Note that the interner may need to reallocate to make space for the new
    /// reference, just the same as a `HashSet` would. This cost is amortized to
    /// `O(1)` as it is in other standard library collections.
    ///
    /// If you have an owned item (especially if it has a cheap transformation
    /// to `Box`) and no longer need the ownership, pass it in directly.
    /// Otherwise, pass in a reference.
    ///
    /// See `get` for more about the interned symbol.
    pub fn intern<R>(&self, t: R) -> Interned<'_, T>
    where
        R: Borrow<T> + Into<Box<T>>,
    {
        let borrowed = t.borrow();
        if let Some(interned) = self.get(borrowed) {
            return interned;
        }

        let mut arena = self
            .arena
            .write()
            .expect("interner lock should not be poisoned");

        // If someone interned the item between the above check and us acquiring
        // the write lock, this heap allocation isn't necessary. However, this
        // is expected to be rare, so we don't bother with doing another lookup
        // before creating the box. Using the raw_entry API could avoid this,
        // but needs a different call than intern_raw to use the intrinsic
        // BuildHasher rather than an external one. It's not worth the effort.

        let entry = arena.entry(PinBox::new(t.into()));
        #[allow(unsafe_code)] // SAFETY: Interned ties the lifetime to the interner.
        match entry {
            Entry::Occupied(entry) => Interned(unsafe { entry.key().as_ref() }),
            Entry::Vacant(entry) => {
                let interned = Interned(unsafe { entry.key().as_ref() });
                entry.insert(());
                interned
            },
        }
    }

    /// Get an interned reference out of this interner.
    ///
    /// The returned reference is bound to the lifetime of the borrow used for
    /// this method. This guarantees that the returned reference will live no
    /// longer than this interner does.
    pub fn get(&self, t: &T) -> Option<Interned<'_, T>> {
        #[allow(unsafe_code)] // SAFETY: Interned ties the lifetime to the interner.
        self.arena
            .read()
            .expect("interner lock should not be poisoned")
            .get_key_value(t)
            .map(|(t, _)| Interned(unsafe { t.as_ref() }))
    }
}

#[allow(unsafe_code)]
#[cfg(feature = "raw")]
impl<T: ?Sized, S> Interner<T, S> {
    /// Raw interning interface for any `T`.
    pub fn intern_raw<Q>(
        &self,
        it: Q,
        hash: u64,
        mut is_match: impl FnMut(&Q, &T) -> bool,
        do_hash: impl Fn(&T) -> u64,
        commit: impl FnOnce(Q) -> Box<T>,
    ) -> Interned<'_, T> {
        if let Some(interned) = self.get_raw(hash, |t| is_match(&it, t)) {
            return interned;
        }

        let mut arena = self
            .arena
            .write()
            .expect("interner lock should not be poisoned");

        match arena.raw_entry_mut().from_hash(hash, |t| is_match(&it, t)) {
            RawEntryMut::Occupied(entry) => Interned(unsafe { entry.key().as_ref() }),
            RawEntryMut::Vacant(entry) => {
                let boxed = PinBox::new(commit(it));
                let interned = Interned(unsafe { boxed.as_ref() });
                entry.insert_with_hasher(hash, boxed, (), |t| do_hash(t));
                interned
            },
        }
    }

    /// Raw interned reference lookup.
    pub fn get_raw(
        &self,
        hash: u64,
        mut is_match: impl FnMut(&T) -> bool,
    ) -> Option<Interned<'_, T>> {
        self.arena
            .read()
            .expect("interner lock should not be poisoned")
            .raw_entry()
            .from_hash(hash, |t| is_match(t))
            .map(|(t, _)| Interned(unsafe { t.as_ref() }))
    }
}

impl<T: ?Sized> Interner<T> {
    /// Create an empty interner.
    ///
    /// The backing set is initially created with a capacity of 0,
    /// so it will not allocate until it is first inserted into.
    pub fn new() -> Self {
        Interner {
            arena: RwLock::new(HashMap::default()),
        }
    }

    /// Create an empty interner with the specified capacity.
    ///
    /// The interner will be able to hold at least `capacity` items without reallocating.
    /// If `capacity` is 0, the interner will not initially allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Interner {
            arena: RwLock::new(HashMap::with_capacity_and_hasher(
                capacity,
                RandomState::default(),
            )),
        }
    }
}

/// Constructors to control the backing `HashSet`'s hash function
impl<T: ?Sized, H: BuildHasher> Interner<T, H> {
    #[cfg(not(feature = "hashbrown"))]
    /// Create an empty interner which will use the given hasher to hash the values.
    ///
    /// The interner is also created with the default capacity.
    pub fn with_hasher(hasher: H) -> Self {
        Interner {
            arena: RwLock::new(HashMap::with_hasher(hasher)),
        }
    }

    #[cfg(feature = "hashbrown")]
    /// Create an empty interner which will use the given hasher to hash the values.
    ///
    /// The interner is also created with the default capacity.
    pub const fn with_hasher(hasher: H) -> Self {
        Interner {
            arena: RwLock::new(HashMap::with_hasher(hasher)),
        }
    }

    /// Create an empty interner with the specified capacity, using `hasher` to hash the values.
    ///
    /// The interner will be able to hold at least `capacity` items without reallocating.
    /// If `capacity` is 0, the interner will not initially allocate.
    pub fn with_capacity_and_hasher(capacity: usize, hasher: H) -> Self {
        Interner {
            arena: RwLock::new(HashMap::with_capacity_and_hasher(capacity, hasher)),
        }
    }
}
