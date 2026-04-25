use crate::models::NodeEntry;

pub(in crate::ui) enum RefreshResult {
    Loaded(Vec<NodeEntry>),
    Failed(String),
}
