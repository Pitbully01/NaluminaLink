use crate::models::NodeEntry;

#[derive(Clone, Copy, Debug)]
pub(in crate::ui) enum RefreshErrorSource {
    NodeDiscovery,
}

pub(in crate::ui) struct RefreshError {
    pub source: RefreshErrorSource,
    pub message: String,
}

impl RefreshError {
    pub(in crate::ui) fn node_discovery(message: String) -> Self {
        Self {
            source: RefreshErrorSource::NodeDiscovery,
            message,
        }
    }
}

pub(in crate::ui) enum RefreshResult {
    Loaded(Vec<NodeEntry>),
    Failed(RefreshError),
}
