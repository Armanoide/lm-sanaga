use sn_core::error::{Error, Result};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait RwLockExt<T> {
    fn read_lock(&self, context: &str) -> Result<RwLockReadGuard<'_, T>>;
    fn read_lock_mut(&self, context: &str) -> Result<RwLockReadGuard<'_, T>>;
    fn write_lock_mut(&self, context: &str) -> Result<RwLockWriteGuard<'_, T>>;
    fn write_lock(&self, context: &str) -> Result<RwLockWriteGuard<'_, T>>;
}

pub trait RwLockExtOpt<T> {
    fn read_lock(&self, context: &str) -> Result<Option<RwLockReadGuard<'_, T>>>;
    fn read_lock_mut(&self, context: &str) -> Result<Option<RwLockReadGuard<'_, T>>>;
    fn write_lock_mut(&self, context: &str) -> Result<Option<RwLockWriteGuard<'_, T>>>;
    fn write_lock(&self, context: &str) -> Result<Option<RwLockWriteGuard<'_, T>>>;
}

impl<T> RwLockExt<T> for Arc<RwLock<T>> {
    fn read_lock(&self, context: &str) -> Result<RwLockReadGuard<'_, T>> {
        self.read()
            .map_err(|e| Error::CacheLockPoisoned(format!("[{}] {}", context, e)))
    }

    fn read_lock_mut(&self, context: &str) -> Result<RwLockReadGuard<'_, T>> {
        self.read()
            .map_err(|e| Error::CacheLockPoisoned(format!("[{}] {}", context, e)))
    }

    fn write_lock_mut(&self, context: &str) -> Result<RwLockWriteGuard<'_, T>> {
        self.write()
            .map_err(|e| Error::CacheLockPoisoned(format!("[{}] {}", context, e)))
    }

    fn write_lock(&self, context: &str) -> Result<RwLockWriteGuard<'_, T>> {
        self.write()
            .map_err(|e| Error::CacheLockPoisoned(format!("[{}] {}", context, e)))
    }
}

impl<T> RwLockExtOpt<T> for Option<&Arc<RwLock<T>>> {
    fn read_lock(&self, context: &str) -> Result<Option<RwLockReadGuard<'_, T>>> {
        match self {
            Some(lock) => Ok(Some(lock.read_lock(context)?)),
            None => Err(Error::CacheLockPoisoned(context.to_string())),
        }
    }

    fn read_lock_mut(&self, context: &str) -> Result<Option<RwLockReadGuard<'_, T>>> {
        match self {
            Some(lock) => Ok(Some(lock.read_lock_mut(context)?)),
            None => Err(Error::CacheLockPoisoned(context.to_string())),
        }
    }

    fn write_lock_mut(&self, context: &str) -> Result<Option<RwLockWriteGuard<'_, T>>> {
        match self {
            Some(lock) => Ok(Some(lock.write_lock_mut(context)?)),
            None => Err(Error::CacheLockPoisoned(context.to_string())),
        }
    }

    fn write_lock(&self, context: &str) -> Result<Option<RwLockWriteGuard<'_, T>>> {
        match self {
            Some(lock) => Ok(Some(lock.write_lock(context)?)),
            None => Err(Error::CacheLockPoisoned(context.to_string())),
        }
    }
}
