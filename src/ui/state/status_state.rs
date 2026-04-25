use crate::i18n::I18n;
use crate::ui::refresh::{RefreshError, RefreshErrorSource};

pub(in crate::ui) struct UiStatus {
    message: String,
}

impl UiStatus {
    pub(in crate::ui) fn new(i18n: &I18n) -> Self {
        Self {
            message: i18n.text("status.ready"),
        }
    }

    pub(in crate::ui) fn set_refreshing(&mut self, i18n: &I18n) {
        self.message = i18n.text("status.refreshing_nodes");
    }

    pub(in crate::ui) fn set_loaded_nodes(&mut self, i18n: &I18n, count: usize) {
        self.message = i18n.text_with("status.loaded_nodes", &[("count", count.to_string())]);
    }

    pub(in crate::ui) fn set_refresh_failed(&mut self, i18n: &I18n, error: RefreshError) {
        let source = match error.source {
            RefreshErrorSource::NodeDiscovery => "node_discovery",
        };
        let display = format!("[{source}] {}", error.message);

        self.message = i18n.text_with("status.refresh_failed", &[("error", display)]);
    }

    pub(in crate::ui) fn set_refresh_disconnected(&mut self, i18n: &I18n) {
        self.message = i18n.text("status.refresh_disconnected");
    }

    pub(in crate::ui) fn set_doctor_message(&mut self, i18n: &I18n) {
        self.message = i18n.text("doctor.message");
    }

    pub(in crate::ui) fn text(&self) -> &str {
        &self.message
    }
}
