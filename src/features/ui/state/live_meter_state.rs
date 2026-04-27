use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::features::node_discovery::{sample_source_levels, NodeEntry};

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::features::ui) struct MeterLevels {
    pub left: f32,
    pub right: f32,
}

struct MeterWorker {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

pub(in crate::features::ui) struct LiveMeterStore {
    readings: Arc<Mutex<BTreeMap<u32, MeterLevels>>>,
    workers: BTreeMap<u32, MeterWorker>,
}

impl LiveMeterStore {
    pub(in crate::features::ui) fn new() -> Self {
        Self {
            readings: Arc::new(Mutex::new(BTreeMap::new())),
            workers: BTreeMap::new(),
        }
    }

    fn stop_worker(&mut self, node_id: u32) {
        let Some(worker) = self.workers.remove(&node_id) else {
            return;
        };

        worker.stop.store(true, Ordering::Relaxed);
        let _ = worker.handle.join();
        if let Ok(mut readings) = self.readings.lock() {
            readings.remove(&node_id);
        }
    }

    fn start_worker(&mut self, node: &NodeEntry) {
        if self.workers.contains_key(&node.id) {
            return;
        }

        let readings = Arc::clone(&self.readings);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let node_id = node.id;
        let channels_hint = node.channels_hint;

        let handle = thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                if let Some((left, right)) = sample_source_levels(node_id, channels_hint) {
                    if let Ok(mut state) = readings.lock() {
                        state.insert(node_id, MeterLevels { left, right });
                    }
                }

                thread::sleep(Duration::from_millis(50));
            }
        });

        self.workers.insert(node.id, MeterWorker { stop, handle });
    }

    pub(in crate::features::ui) fn sync_sources<'a, I>(&mut self, nodes: &[NodeEntry], source_ids: I)
    where
        I: IntoIterator<Item = u32>,
    {
        let desired: BTreeSet<u32> = source_ids.into_iter().collect();
        let existing: Vec<u32> = self.workers.keys().copied().collect();

        for node_id in existing {
            if !desired.contains(&node_id) {
                self.stop_worker(node_id);
            }
        }

        for node_id in desired {
            let Some(node) = nodes.iter().find(|entry| entry.id == node_id) else {
                continue;
            };

            self.start_worker(node);
        }
    }

    pub(in crate::features::ui) fn reading(&self, node_id: u32) -> Option<MeterLevels> {
        self.readings
            .lock()
            .ok()
            .and_then(|readings| readings.get(&node_id).copied())
    }
}

impl Drop for LiveMeterStore {
    fn drop(&mut self) {
        let worker_ids: Vec<u32> = self.workers.keys().copied().collect();
        for node_id in worker_ids {
            self.stop_worker(node_id);
        }
    }
}