use CborConv;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LayerShellPermissions {
    pub background: bool,
    pub bottom: bool,
    pub top: bool,
    pub overlay: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Permissions {
    pub layer_shell: LayerShellPermissions,
    pub private_api: bool,
}

impl CborConv for Permissions {}
