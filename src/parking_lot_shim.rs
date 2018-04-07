#[cfg(feature = "parking_lot")]
use parking_lot::RwLockReadGuard;
#[cfg(not(feature = "parking_lot"))]
use std::sync::{RwLock, RwLockWriteGuard};

#[cfg(not(feature = "parking_lot"))]
pub(crate) trait RwLockUpgradableReadShim<T> {
    fn upgradable_read(&self) -> RwLockWriteGuard<T>;
}

#[cfg(not(feature = "parking_lot"))]
impl<T> RwLockUpgradableReadShim<T> for RwLock<T> {
    fn upgradable_read(&self) -> RwLockWriteGuard<T> {
        self.write().unwrap()
    }
}

#[cfg(not(feature = "parking_lot"))]
pub(crate) trait RwLockGuardUpgradeShim<'a, T> {
    fn upgrade(self) -> RwLockWriteGuard<'a, T>;
}

#[cfg(not(feature = "parking_lot"))]
impl<'a, T> RwLockGuardUpgradeShim<'a, T> for RwLockWriteGuard<'a, T> {
    fn upgrade(self) -> Self {
        self
    }
}

#[cfg(feature = "parking_lot")]
pub(crate) trait LockResult<T> {
    fn unwrap(self) -> T;
}

#[cfg(feature = "parking_lot")]
impl<'a, T> LockResult<Self> for RwLockReadGuard<'a, T> {
    fn unwrap(self) -> Self {
        self
    }
}
