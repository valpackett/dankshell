use parking_lot::{RwLock, RwLockReadGuard, MappedRwLockReadGuard, RwLockWriteGuard, MappedRwLockWriteGuard};

/// Like https://github.com/tyleo/mut_static but with
/// - const fn creation thanks to parking_lot
/// - panic when reading instead of returning Result
///
/// For things that can be initialized statically, just use RwLock directly
pub struct MutStatic<T> {
    data: RwLock<Option<T>>,
}

unsafe impl<T> Sync for MutStatic<T> where T: Sync {}

impl<T> MutStatic<T> {
    pub const fn new() -> MutStatic<T> {
        MutStatic {
            data: RwLock::new(None),
        }
    }

    pub fn read(&self) -> MappedRwLockReadGuard<T> {
        RwLockReadGuard::map(self.data.read(), |x| x.as_ref().expect("MutStatic not set yet"))
    }

    pub fn write(&self) -> MappedRwLockWriteGuard<T> {
        RwLockWriteGuard::map(self.data.write(), |x| x.as_mut().expect("MutStatic not set yet"))
    }

    pub fn set(&self, obj: T) {
        *self.data.write() = Some(obj);
    }
}
