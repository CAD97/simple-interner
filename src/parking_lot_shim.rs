//! Allows using parking_lot as if it poisoned like std does.

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

pub(crate) trait LockResult<T> {
    fn expect(self, msg: &str) -> T;
}

impl<'a, T> LockResult<Self> for RwLockReadGuard<'a, T> {
    fn expect(self, _msg: &str) -> Self {
        self
    }
}

impl<'a, T> LockResult<Self> for RwLockWriteGuard<'a, T> {
    fn expect(self, _msg: &str) -> Self {
        self
    }
}
