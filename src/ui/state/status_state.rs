use crate::i18n::I18n;
use crate::ui::refresh::{RefreshError, RefreshErrorSource};
use crate::ui::state::StatusKey;

pub(in crate::ui) struct UiStatus {
    message: String,
}

impl UiStatus {
    fn resolve_text(i18n: &I18n, key: StatusKey) -> String {
        i18n.text(key.as_str())
    }

    fn resolve_text_with(i18n: &I18n, key: StatusKey, placeholders: &[(&str, String)]) -> String {
        i18n.text_with(key.as_str(), placeholders)
    }

    pub(in crate::ui) fn new(i18n: &I18n) -> Self {
        Self {
            message: Self::resolve_text(i18n, StatusKey::Ready),
        }
    }

    pub(in crate::ui) fn set_refreshing(&mut self, i18n: &I18n) {
        self.message = Self::resolve_text(i18n, StatusKey::RefreshingNodes);
    }

    pub(in crate::ui) fn set_loaded_nodes(&mut self, i18n: &I18n, count: usize) {
        self.message = Self::resolve_text_with(
            i18n,
            StatusKey::LoadedNodes,
            &[("count", count.to_string())],
        );
    }

    pub(in crate::ui) fn set_refresh_failed(&mut self, i18n: &I18n, error: RefreshError) {
        let source_label = match error.source {
            RefreshErrorSource::NodeDiscovery => {
                Self::resolve_text(i18n, StatusKey::RefreshErrorSourceNodeDiscovery)
            }
        };
        let display = format!("{source_label}: {}", error.message);

        self.message =
            Self::resolve_text_with(i18n, StatusKey::RefreshFailed, &[("error", display)]);
    }

    pub(in crate::ui) fn set_refresh_disconnected(&mut self, i18n: &I18n) {
        self.message = Self::resolve_text(i18n, StatusKey::RefreshDisconnected);
    }

    pub(in crate::ui) fn set_scene_applied(&mut self, i18n: &I18n, preset: String) {
        self.message =
            Self::resolve_text_with(i18n, StatusKey::SceneApplied, &[("preset", preset)]);
    }

    pub(in crate::ui) fn set_doctor_message(&mut self, i18n: &I18n) {
        self.message = Self::resolve_text(i18n, StatusKey::DoctorMessage);
    }

    pub(in crate::ui) fn text(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::UiStatus;
    use crate::i18n::I18n;
    use crate::ui::refresh::RefreshError;

    fn i18n_en() -> I18n {
        std::env::set_var("NALUMINALINK_LANG", "en");
        I18n::load()
    }

    #[test]
    fn new_status_starts_with_ready_message() {
        let i18n = i18n_en();
        let status = UiStatus::new(&i18n);
        assert_eq!(status.text(), "Ready.");
    }

    #[test]
    fn loaded_nodes_replaces_placeholder_count() {
        let i18n = i18n_en();
        let mut status = UiStatus::new(&i18n);
        status.set_loaded_nodes(&i18n, 42);
        assert!(status.text().contains("42"));
        assert!(!status.text().contains("{{count}}"));
    }

    #[test]
    fn refresh_failed_includes_localized_source_and_error_text() {
        let i18n = i18n_en();
        let mut status = UiStatus::new(&i18n);
        let error = RefreshError::node_discovery("pw core disconnected".to_string());
        status.set_refresh_failed(&i18n, error);

        assert!(status.text().contains("Node discovery"));
        assert!(status.text().contains("pw core disconnected"));
    }

    #[test]
    fn scene_applied_status_contains_selected_preset_name() {
        let i18n = i18n_en();
        let mut status = UiStatus::new(&i18n);
        status.set_scene_applied(&i18n, "Monitor Focus".to_string());

        assert!(status.text().contains("Monitor Focus"));
    }
}
