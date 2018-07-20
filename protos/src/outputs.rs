//! Structures describing display outputs/heads.
//! Why not use wl_output?
//! Well, we need to expose internal stuff anyway (extra_scale, heck, the whole head/output separation)
//! So why bother tying these to wl_output...

use CborConv;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OutputInfo {
    pub id: u32,
    // Modifiers
    pub transform: u32,
    pub scale: i32,
    pub extra_scale: f32,
    // Rect in compositor space
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadInfo {
    // Connector
    pub name: String,
    // Attached monitor/projector/whatever
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub mm_width: i32,
    pub mm_height: i32,
    pub internal: bool,
    // Status
    pub connected: bool,
    pub enabled: bool,
    pub device_changed: bool,
    // What is displayed on it
    pub output: Option<OutputInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputState {
    pub heads: Vec<HeadInfo>,
}

impl CborConv for OutputState {}
