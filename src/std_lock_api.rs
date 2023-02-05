//! Implements the [`StdRawRwLock`] helper type which wraps a
//! [`std::syn::RwLock`] so that it can be used with `lock_api`

use {
    core::ops::Deref,
    std::{
        cell::UnsafeCell,
        fmt,
        mem::MaybeUninit,
        pin::Pin,
        sync::{Once, RwLock, RwLockWriteGuard, TryLockError},
    },
};

use lock_api::RawRwLock;

pub struct StdRawRwLock {
    lock: LazyPinBox<RwLock<()>>,
    write: UnsafeCell<Option<RwLockWriteGuard<'static, ()>>>,
}

#[allow(unsafe_code)]
// Safety:
// - RwLockWriteGuard is !Send
// - write is only accessed through RawRwLock while an exclusive lock is held
// - RawRwLock::GuardMarker is set to GuardNoSend
// - StdRawRwLock can be Send since the guard exposed through RawRwLock is !Send
unsafe impl Send for StdRawRwLock {}

#[allow(unsafe_code)]
// Safety:
// - UnsafeCell is !Sync
// - write is only accessed through RawRwLock while an exclusive lock is held
// - thus StdRawRwLock can be sync
unsafe impl Sync for StdRawRwLock {}

impl fmt::Debug for StdRawRwLock {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("StdRawRwLock").finish()
    }
}

impl Drop for StdRawRwLock {
    fn drop(&mut self) {
        // Drop any outstanding write guard before the lock
        std::mem::drop(self.write.get_mut().take());
    }
}

#[allow(unsafe_code)]
// Safety: std::sync::RwLock fulfills the safety contract of RawRwLock
unsafe impl RawRwLock for StdRawRwLock {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = StdRawRwLock {
        lock: LazyPinBox::new(),
        write: UnsafeCell::new(None),
    };

    type GuardMarker = lock_api::GuardNoSend;

    fn lock_shared(&self) {
        let guard = self
            .lock
            .read()
            .expect("interner lock should not be poisoned");

        // The read guard can be reconstructed later from another read
        std::mem::forget(guard);
    }

    fn try_lock_shared(&self) -> bool {
        let guard = match self.lock.try_read() {
            Ok(guard) => guard,
            Err(TryLockError::WouldBlock) => return false,
            r @ Err(TryLockError::Poisoned(_)) => r.expect("interner lock should not be poisoned"),
        };

        // The read guard can be reconstructed later from another read
        std::mem::forget(guard);

        true
    }

    unsafe fn unlock_shared(&self) {
        // Since this method may only be called if a shared lock is held,
        // this if branch should always be taken
        if let Ok(new_guard) = self.lock.try_read() {
            // Safety:
            // - unlock_shared may only be called if a shared lock is held
            //   in the current context
            // - thus an old guard for that shared lock must exist
            let old_guard = std::ptr::read(&new_guard);

            std::mem::drop(old_guard);
            std::mem::drop(new_guard);
        }
    }

    fn lock_exclusive<'a>(&'a self) {
        let guard: RwLockWriteGuard<'a, ()> = self
            .lock
            .write()
            .expect("interner lock should not be poisoned");
        // Safety: lifetime erasure to store a never-exposed reference to self
        let guard: RwLockWriteGuard<'static, ()> = unsafe { std::mem::transmute(guard) };

        // Safety:
        // - the RwLock is pinned inside the LazyPinBox,
        //   so any references to it remain valid
        // - the interior mutability write occurs while the unique
        //   write lock guard is held
        unsafe { *self.write.get() = Some(guard) };
    }

    fn try_lock_exclusive<'a>(&'a self) -> bool {
        let guard: RwLockWriteGuard<'a, ()> = match self.lock.try_write() {
            Ok(guard) => guard,
            Err(TryLockError::WouldBlock) => return false,
            r @ Err(TryLockError::Poisoned(_)) => r.expect("interner lock should not be poisoned"),
        };
        // Safety: lifetime erasure to store a never-exposed reference to self
        let guard: RwLockWriteGuard<'static, ()> = unsafe { std::mem::transmute(guard) };

        // Safety:
        // - the RwLock is pinned inside the LazyPinBox,
        //   so any references to it remain valid
        // - the interior mutability write occurs while the unique
        //   write lock guard is held
        unsafe { *self.write.get() = Some(guard) };

        true
    }

    unsafe fn unlock_exclusive<'a>(&'a self) {
        // Safety:
        // - unlock_exclusive may only be called if an exclusive lock is held
        //   in the current context -> the if branch is taken
        // - the interior mutability write occurs while the unique
        //   write lock guard is held
        if let Some(guard) = unsafe { (*self.write.get()).take() } {
            // Safety: lifetime un-erasure for the self-referential guard
            let guard: RwLockWriteGuard<'a, ()> = unsafe { std::mem::transmute(guard) };

            std::mem::drop(guard);
        }
    }
}

struct LazyPinBox<T: Default> {
    init: Once,
    data: UnsafeCell<MaybeUninit<Pin<Box<T>>>>,
}

#[allow(unsafe_code)]
// Safety:
// - UnsafeCell is !Sync
// - every shared read access to data goes through deref
// - every deref goes through call_once first
// - thus no read access can occur before the first write has succeeded
// - call_once provides unique access, so only one write occurs
unsafe impl<T: Default + Sync> Sync for LazyPinBox<T> {}

impl<T: Default> LazyPinBox<T> {
    const fn new() -> Self {
        Self {
            init: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<T: Default> Drop for LazyPinBox<T> {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        // Safety:
        // - if init completed, data is initialised and must be dropped
        // - if init did not complete, data may be uninitialised
        if self.init.is_completed() {
            unsafe { (*self.data.get()).assume_init_drop() };
        }
    }
}

impl<T: Default> Deref for LazyPinBox<T> {
    type Target = T;

    #[allow(unsafe_code)]
    fn deref(&self) -> &Self::Target {
        // Safety:
        // - every deref goes through call_once first
        // - thus no read access can occur before the first write has succeeded
        // - call_once provides unique access, so only one write occurs
        self.init.call_once(|| unsafe {
            (*self.data.get()).write(Box::pin(T::default()));
        });

        // Safety: data has been initialised exactly once before
        unsafe { (*self.data.get()).assume_init_ref() }
    }
}

#[cfg(test)]
mod tests {
    use super::LazyPinBox;

    #[test]
    #[allow(dead_code)]
    fn send_sync_unpin() {
        fn is_send<T: Send>(_t: &T) {}
        fn is_sync<T: Sync>(_t: &T) {}
        fn is_unpin<T: Unpin>(_t: &T) {}

        fn is_lazy_pin_box_send<T: Default + Send>(t: &LazyPinBox<T>) {
            is_send(t)
        }

        fn is_lazy_pin_box_sync<T: Default + Sync>(t: &LazyPinBox<T>) {
            is_sync(t)
        }

        fn is_lazy_pin_box_unpin<T: Default>(t: &LazyPinBox<T>) {
            is_unpin(t)
        }
    }
}
