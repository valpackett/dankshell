use std::mem;
use parking_lot::RwLock;
use weston_rs::{libweston_sys, Surface, ForeignType};

pub static SURFACES: RwLock<Vec<SurfaceListItem>> = RwLock::new(Vec::new());

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

// TODO: consider fragile::Sticky
unsafe impl Send for SurfaceListItem {}
unsafe impl Sync for SurfaceListItem {}
