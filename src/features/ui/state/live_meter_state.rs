use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::features::node_discovery::{sample_source_levels, NodeEntry};

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::features::ui) struct MeterLevels {
    pub left: f32,
    pub right: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::features::ui) struct MeterSnapshot {
    pub current: MeterLevels,
    pub peak: MeterLevels,
}

struct MeterWorker {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

pub(in crate::features::ui) struct LiveMeterStore {
    readings: Arc<Mutex<BTreeMap<u32, MeterSnapshot>>>,
    workers: BTreeMap<u32, MeterWorker>,
}

impl LiveMeterStore {
    const POLL_INTERVAL: Duration = Duration::from_millis(16);
    const ATTACK: f32 = 0.28;
    const RELEASE: f32 = 0.08;
    const PEAK_HOLD: Duration = Duration::from_secs(3);

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
            let mut smoothed = MeterLevels {
                left: 0.0,
                right: 0.0,
            };
            let mut peak = MeterLevels {
                left: 0.0,
                right: 0.0,
            };
            let mut peak_expiry = Instant::now();

            while !stop_flag.load(Ordering::Relaxed) {
                if let Some((left, right)) = sample_source_levels(node_id, channels_hint) {
                    let mut new_peak = false;

                    smoothed.left = smooth_level(smoothed.left, left);
                    smoothed.right = smooth_level(smoothed.right, right);

                    if smoothed.left >= peak.left {
                        peak.left = smoothed.left;
                        new_peak = true;
                    }
                    if smoothed.right >= peak.right {
                        peak.right = smoothed.right;
                        new_peak = true;
                    }

                    if new_peak {
                        peak_expiry = Instant::now() + Self::PEAK_HOLD;
                    }

                    if let Ok(mut state) = readings.lock() {
                        state.insert(
                            node_id,
                            MeterSnapshot {
                                current: smoothed,
                                peak,
                            },
                        );
                    }
                }

                if Instant::now() >= peak_expiry {
                    peak.left = smoothed.left;
                    peak.right = smoothed.right;
                }

                thread::sleep(Self::POLL_INTERVAL);
            }
        });

        self.workers.insert(node.id, MeterWorker { stop, handle });
    }

    pub(in crate::features::ui) fn sync_sources<'a, I>(
        &mut self,
        nodes: &[NodeEntry],
        source_ids: I,
    ) where
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

    pub(in crate::features::ui) fn reading(&self, node_id: u32) -> Option<MeterSnapshot> {
        self.readings
            .lock()
            .ok()
            .and_then(|readings| readings.get(&node_id).copied())
    }
}

fn smooth_level(previous: f32, input: f32) -> f32 {
    if input > previous {
        previous + (input - previous) * LiveMeterStore::ATTACK
    } else {
        previous + (input - previous) * LiveMeterStore::RELEASE
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
