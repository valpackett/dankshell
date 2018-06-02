use weston_rs::{View, Resize};

/// Per-surface data (libweston-desktop)
pub struct SurfaceContext {
    pub view: View,
    pub focus_count: i16,

    // Resize stuff
    pub resize_edges: Resize,
    pub last_width: f32,
    pub last_height: f32,
}
