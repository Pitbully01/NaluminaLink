pub(in crate::ui) enum StatusKey {
    Ready,
    RefreshingNodes,
    LoadedNodes,
    RefreshFailed,
    RefreshDisconnected,
    SceneApplied,
    DoctorMessage,
    RefreshErrorSourceNodeDiscovery,
}

impl StatusKey {
    pub(in crate::ui) fn as_str(&self) -> &'static str {
        match self {
            StatusKey::Ready => "status.ready",
            StatusKey::RefreshingNodes => "status.refreshing_nodes",
            StatusKey::LoadedNodes => "status.loaded_nodes",
            StatusKey::RefreshFailed => "status.refresh_failed",
            StatusKey::RefreshDisconnected => "status.refresh_disconnected",
            StatusKey::SceneApplied => "status.scene_applied",
            StatusKey::DoctorMessage => "doctor.message",
            StatusKey::RefreshErrorSourceNodeDiscovery => {
                "status.refresh_error_source.node_discovery"
            }
        }
    }
}
