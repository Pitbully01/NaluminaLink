use std::sync::mpsc;

use log::{debug, error, info};

use super::super::NaluminaApp;
use super::{RefreshError, RefreshResult};
use crate::features::node_discovery::NodeEntry;

impl NaluminaApp {
    fn on_refresh_success(&mut self, nodes: Vec<NodeEntry>) {
        info!("refresh: success with {} nodes", nodes.len());
        self.status.set_loaded_nodes(&self.i18n, nodes.len());
        self.nodes = nodes;
        self.sync_node_defaults();
    }

    fn on_refresh_error(&mut self, error: RefreshError) {
        error!("refresh: failed from {:?}: {}", error.source, error.message);
        self.status.set_refresh_failed(&self.i18n, error);
    }

    fn on_refresh_pending(&mut self, receiver: mpsc::Receiver<RefreshResult>) {
        self.refresh_inflight = Some(receiver);
    }

    fn on_refresh_disconnected(&mut self) {
        error!("refresh: worker channel disconnected");
        self.status.set_refresh_disconnected(&self.i18n);
    }

    pub(in crate::features::ui) fn poll_refresh(&mut self) {
        let Some(receiver) = self.refresh_inflight.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(RefreshResult::Loaded(nodes)) => self.on_refresh_success(nodes),
            Ok(RefreshResult::Failed(error)) => self.on_refresh_error(error),
            Err(mpsc::TryRecvError::Empty) => {
                debug!("refresh: still running");
                self.on_refresh_pending(receiver)
            }
            Err(mpsc::TryRecvError::Disconnected) => self.on_refresh_disconnected(),
        }
    }
}
