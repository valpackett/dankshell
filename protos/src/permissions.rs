use serde_cbor;

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

impl Permissions {
    pub fn from_cbor(data: &[u8]) -> serde_cbor::error::Result<Permissions> {
        serde_cbor::from_slice(data)
    }

    pub fn to_cbor(&self) -> serde_cbor::error::Result<Vec<u8>> {
        serde_cbor::to_vec(self)
    }
}
