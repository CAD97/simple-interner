use Interned;

use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::mem;

#[cfg(feature = "parking_lot")]
use parking_lot::RwLock;
#[cfg(not(feature = "parking_lot"))]
use std::sync::RwLock;

use parking_lot_shim::*;

// The `Interner` loans out references with the same lifetime as its own borrow.
// This guarantees that the loan cannot outlive the interner.
// Because we only ever add to our arena `HashSet`, and do not give mutable references
// to our boxes, the location of our interned items is stable, and this is sound.
// We're treating `Box` as if it were `PinBox` here, so we need to avoid
// using the fn which `PinBox` makes unsafe manually to maintain safety.

/// An interner based on a `HashSet`. See the crate-level docs for more.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Interner<T: Eq + Hash + ?Sized, H: BuildHasher = RandomState> {
    #[cfg_attr(feature = "serde",
               serde(bound(deserialize = "for<'a> HashSet<Box<T>, H>: ::serde::Deserialize<'a>")))]
    arena: RwLock<HashSet<Box<T>, H>>,
}

// Cannot be derived with the BuildHasher generic
impl<T: Eq + Hash + ?Sized> Default for Interner<T, RandomState> {
    fn default() -> Self {
        Interner {
            arena: RwLock::default(),
        }
    }
}

#[inline(always)]
#[cfg_attr(feature = "cargo-clippy", allow(inline_always))]
fn coerce<T>(t: T) -> T {
    t
}

#[allow(unsafe_code)]
/// The interner interface
impl<T: Eq + Hash + ?Sized, H: BuildHasher> Interner<T, H> {
    /// Get an interned item out of this interner, or insert if it doesn't exist.
    /// Takes borrowed or heap-allocated items. If the item has not been previously interned,
    /// it will be `Into::into`ed a `Box` on the heap and cached. Notably, if you
    /// give this fn a `String` or `Vec`, the allocation will shrink but not reallocate.
    ///
    /// Note that the interner may need to reallocate to make space for the new reference,
    /// just the same as a `HashSet<T>` would. This cost is amortized to `O(1)` as it is
    /// in other standard library collections.
    ///
    /// If you have an owned item (especially if it has a cheap transformation to `Box`)
    /// and no longer need the ownership, pass it in directly. Otherwise, just pass in a reference.
    ///
    /// See `get` for more about the interned symbol.
    pub fn get_or_insert<'a, R>(&'a self, t: R) -> Interned<'a, T>
    where
        R: AsRef<T> + Into<Box<T>>,
    {
        if let Some(interned_t) = self.get(t.as_ref()) {
            return interned_t;
        } else {
            let arena = self.arena.upgradable_read();
            // double read to catch add after first check but before acquiring the upgradable lock
            // cannot reuse Self::get here as stdlib upgradable read lock is shimmed as a write lock
            if let Some(interned_t) = arena.get(t.as_ref()) {
                return Interned(unsafe { mem::transmute(coerce::<&T>(interned_t)) });
            } // cannot else because `arena` would still be borrowed
            let mut arena = arena.upgrade();
            let boxed_t: Box<T> = t.into();
            // Get the reference to loan out _after_ boxing up our data
            let t_ref: &'a T = unsafe { mem::transmute(coerce::<&T>(&boxed_t)) };
            arena.insert(boxed_t);
            Interned(t_ref)
        }
    }

    /// Get an interned reference out of this interner.
    ///
    /// The returned reference is bound to the lifetime of the borrow used for this method.
    /// This guarantees that the returned reference will live no longer than this interner does.
    pub fn get<'a>(&'a self, t: &T) -> Option<Interned<'a, T>> {
        self.arena
            .read()
            .unwrap()
            .get(t)
            .map(|t: &Box<T>| unsafe { mem::transmute(coerce::<&T>(t)) })
    }
}

/// Constructors
impl<T: Eq + Hash + ?Sized> Interner<T> {
    /// Create an empty interner.
    ///
    /// The backing set is initially created with a capacity of 0,
    /// so it will not allocate until it is first inserted into.
    pub fn new() -> Self {
        Interner {
            arena: RwLock::new(HashSet::new()),
        }
    }

    /// Create an empty interner with the specified capacity.
    ///
    /// The interner will be able to hold at least `capacity` items without reallocating.
    /// If `capacity` is 0, the interner will not initially allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Interner {
            arena: RwLock::new(HashSet::with_capacity(capacity)),
        }
    }
}

/// Constructors to control the backing `HashSet`'s hash function
impl<T: Eq + Hash + ?Sized, H: BuildHasher> Interner<T, H> {
    /// Create an empty interner which will use the given hasher to hash the strings.
    ///
    /// The interner is also created with the default capacity.
    pub fn with_hasher(hasher: H) -> Self {
        Interner {
            arena: RwLock::new(HashSet::with_hasher(hasher)),
        }
    }

    /// Create an empty interner with the specified capacity, using `hasher` to hash the strings.
    ///
    /// The interner will be able to hold at least `capacity` items without reallocating.
    /// If `capacity` is 0, the interner will not initially allocate.
    pub fn with_capacity_and_hasher(capacity: usize, hasher: H) -> Self {
        Interner {
            arena: RwLock::new(HashSet::with_capacity_and_hasher(capacity, hasher)),
        }
    }
}
