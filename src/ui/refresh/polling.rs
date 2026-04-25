use std::sync::mpsc;

use super::super::NaluminaApp;
use super::{RefreshError, RefreshResult};
use crate::models::NodeEntry;

impl NaluminaApp {
    fn on_refresh_success(&mut self, nodes: Vec<NodeEntry>) {
        self.status.set_loaded_nodes(&self.i18n, nodes.len());
        self.nodes = nodes;
        self.sync_node_defaults();
    }

    fn on_refresh_error(&mut self, error: RefreshError) {
        self.status.set_refresh_failed(&self.i18n, error);
    }

    fn on_refresh_pending(&mut self, receiver: mpsc::Receiver<RefreshResult>) {
        self.refresh_inflight = Some(receiver);
    }

    fn on_refresh_disconnected(&mut self) {
        self.status.set_refresh_disconnected(&self.i18n);
    }

    pub(in crate::ui) fn poll_refresh(&mut self) {
        let Some(receiver) = self.refresh_inflight.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(RefreshResult::Loaded(nodes)) => self.on_refresh_success(nodes),
            Ok(RefreshResult::Failed(error)) => self.on_refresh_error(error),
            Err(mpsc::TryRecvError::Empty) => self.on_refresh_pending(receiver),
            Err(mpsc::TryRecvError::Disconnected) => self.on_refresh_disconnected(),
        }
    }
}
