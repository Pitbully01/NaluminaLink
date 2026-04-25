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

    pub(in crate::ui) fn set_doctor_message(&mut self, i18n: &I18n) {
        self.message = Self::resolve_text(i18n, StatusKey::DoctorMessage);
    }

    pub(in crate::ui) fn text(&self) -> &str {
        &self.message
    }
}
