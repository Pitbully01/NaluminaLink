use std::sync::mpsc;
use std::thread;

use super::super::NaluminaApp;
use crate::node_discovery::collect_nodes;

impl NaluminaApp {
    pub(in crate::ui) fn start_refresh(&mut self) {
        if self.refresh_inflight.is_some() {
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.refresh_inflight = Some(receiver);
        self.status.set_refreshing(&self.i18n);

        thread::spawn(move || {
            let result = collect_nodes().map_err(|error| error.to_string());
            let _ = sender.send(result);
        });
    }
}
