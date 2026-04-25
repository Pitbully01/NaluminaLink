use std::sync::mpsc;
use std::thread;

use log::{debug, error, info};

use super::super::NaluminaApp;
use super::{RefreshError, RefreshResult};
use crate::features::node_discovery::collect_nodes;

impl NaluminaApp {
    pub(in crate::features::ui) fn start_refresh(&mut self) {
        if self.refresh_inflight.is_some() {
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.refresh_inflight = Some(receiver);
        self.status.set_refreshing(&self.i18n);
        info!("refresh: started background node discovery");

        thread::spawn(move || {
            let result = match collect_nodes() {
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
