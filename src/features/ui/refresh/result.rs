use crate::features::node_discovery::NodeEntry;

#[derive(Clone, Copy, Debug)]
pub(in crate::features::ui) enum RefreshErrorSource {
    NodeDiscovery,
}

pub(in crate::features::ui) struct RefreshError {
    pub source: RefreshErrorSource,
    pub message: String,
}

impl RefreshError {
    pub(in crate::features::ui) fn node_discovery(message: String) -> Self {
        Self {
            source: RefreshErrorSource::NodeDiscovery,
            message,
        }
    }
}

pub(in crate::features::ui) enum RefreshResult {
    Loaded(Vec<NodeEntry>),
    Failed(RefreshError),
}
