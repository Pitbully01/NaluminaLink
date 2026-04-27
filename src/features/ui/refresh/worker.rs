use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;

use log::{debug, error};

use super::super::NaluminaApp;
use super::{RefreshError, RefreshResult};
use crate::features::node_discovery::collect_nodes_for_sources;

impl NaluminaApp {
    pub(in crate::features::ui) fn start_refresh(&mut self) {
        self.start_refresh_with_status(true);
    }

    pub(in crate::features::ui) fn start_refresh_silent(&mut self) {
        self.start_refresh_with_status(false);
    }

    fn start_refresh_with_status(&mut self, updates_status: bool) {
        if self.refresh_inflight.is_some() {
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.refresh_inflight = Some(receiver);
        self.refresh_updates_status = updates_status;
        if updates_status {
            self.status.set_refreshing(&self.i18n);
        }
        debug!("refresh: started background node discovery");

        let probed_source_ids: HashSet<u32> = self
            .input_channels
            .iter()
            .filter_map(|channel| channel.source_node_id)
            .collect();

        thread::spawn(move || {
            let result = match collect_nodes_for_sources(&probed_source_ids) {
                Ok(nodes) => {
                    debug!("refresh: discovered {} nodes", nodes.len());
                    RefreshResult::Loaded(nodes)
                }
                Err(error) => {
                    error!("refresh: node discovery failed: {}", error);
                    RefreshResult::Failed(RefreshError::node_discovery(error.to_string()))
                }
            };
            let _ = sender.send(result);
        });
    }
}
