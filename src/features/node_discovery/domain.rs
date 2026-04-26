#[derive(Clone, Debug)]
pub struct NodeEntry {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub volume_hint: Option<f32>,
    pub channels_hint: Option<u8>,
}
