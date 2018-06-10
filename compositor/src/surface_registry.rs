use std::mem;
use mut_static::MutStatic;
use weston_rs::{libweston_sys, Surface, ForeignType};

lazy_static! {
    pub static ref SURFACES: MutStatic<Vec<SurfaceListItem>> = MutStatic::from(Vec::new());
}

pub enum SurfaceListItem {
    Desktop(*mut libweston_sys::weston_desktop_surface),
    LayerShell(mem::ManuallyDrop<Surface>), // XXX: never deleted right now
}

impl PartialEq for SurfaceListItem {
    fn eq(&self, other: &SurfaceListItem) -> bool {
        use self::SurfaceListItem::*;
        match (self, other) {
            (Desktop(a), Desktop(b)) => a == b,
            (LayerShell(a), LayerShell(b)) => a.as_ptr() == b.as_ptr(),
            _ => false,
        }
    }
}

unsafe impl Sync for SurfaceListItem {}
