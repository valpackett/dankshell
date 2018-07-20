use std::{str, ffi};
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

pub fn opt_cstr_to_string(x: Option<&ffi::CStr>) -> Option<String> {
    x.map(|cs| cs.to_bytes()).and_then(|bs| str::from_utf8(bs).ok()).map(|s| s.to_owned())
}

/// Use when you're *sure* it's a valid utf8 string.
pub fn cstr_to_string(x: &ffi::CStr) -> String {
    str::from_utf8(x.to_bytes()).unwrap().to_owned()
}
