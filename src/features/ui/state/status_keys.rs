pub(in crate::features::ui) enum StatusKey {
    Ready,
    RefreshingNodes,
    LoadedNodes,
    RefreshFailed,
    RefreshDisconnected,
    DoctorMessage,
    RefreshErrorSourceNodeDiscovery,
}

impl StatusKey {
    pub(in crate::features::ui) fn as_str(&self) -> &'static str {
        match self {
            StatusKey::Ready => "status.ready",
            StatusKey::RefreshingNodes => "status.refreshing_nodes",
            StatusKey::LoadedNodes => "status.loaded_nodes",
            StatusKey::RefreshFailed => "status.refresh_failed",
            StatusKey::RefreshDisconnected => "status.refresh_disconnected",
            StatusKey::DoctorMessage => "doctor.message",
            StatusKey::RefreshErrorSourceNodeDiscovery => {
                "status.refresh_error_source.node_discovery"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StatusKey;

    #[test]
    fn status_keys_map_to_expected_translation_ids() {
        assert_eq!(StatusKey::Ready.as_str(), "status.ready");
        assert_eq!(StatusKey::RefreshingNodes.as_str(), "status.refreshing_nodes");
        assert_eq!(StatusKey::LoadedNodes.as_str(), "status.loaded_nodes");
        assert_eq!(StatusKey::RefreshFailed.as_str(), "status.refresh_failed");
        assert_eq!(StatusKey::RefreshDisconnected.as_str(), "status.refresh_disconnected");
        assert_eq!(StatusKey::DoctorMessage.as_str(), "doctor.message");
        assert_eq!(
            StatusKey::RefreshErrorSourceNodeDiscovery.as_str(),
            "status.refresh_error_source.node_discovery"
        );
    }
}
