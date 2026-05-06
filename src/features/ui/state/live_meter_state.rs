use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};

use crate::features::node_discovery::NodeEntry;

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
        let channels_hint = node.channels_hint.unwrap_or(2).clamp(1, 2);
        let sample_frame_size = usize::from(channels_hint) * 2 * 128;

        let handle = thread::spawn(move || {
            let mut child = match spawn_meter_process(node_id, channels_hint) {
                Some(child) => child,
                None => return,
            };

            let Some(stdout) = child.stdout.take() else {
                let _ = child.kill();
                let _ = child.wait();
                return;
            };

            let mut reader = BufReader::new(stdout);
            let mut smoothed = MeterLevels::default();
            let mut peak = MeterLevels::default();
            let mut buffer = vec![0_u8; sample_frame_size.max(256)];

            while !stop_flag.load(Ordering::Relaxed) {
                let bytes_read = match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) => count,
                    Err(_) => break,
                };

                let (left, right) = decode_levels(&buffer[..bytes_read], channels_hint);
                smoothed.left = smooth_level(smoothed.left, left);
                smoothed.right = smooth_level(smoothed.right, right);
                peak.left = peak.left.max(smoothed.left);
                peak.right = peak.right.max(smoothed.right);

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

            let _ = child.kill();
            let _ = child.wait();
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

fn spawn_meter_process(node_id: u32, channels: u8) -> Option<Child> {
    Command::new("pw-cat")
        .args([
            "--record",
            "--raw",
            "--target",
            &node_id.to_string(),
            "--format",
            "s16",
            "--channels",
            &channels.to_string(),
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn decode_levels(buffer: &[u8], channels: u8) -> (f32, f32) {
    let stride = usize::from(channels).max(1) * 2;
    let mut left_peak = 0.0_f32;
    let mut right_peak = 0.0_f32;

    for frame in buffer.chunks_exact(stride) {
        let left = i16::from_le_bytes([frame[0], frame[1]]) as f32 / i16::MAX as f32;
        left_peak = left_peak.max(left.abs());

        if channels > 1 {
            let right = i16::from_le_bytes([frame[2], frame[3]]) as f32 / i16::MAX as f32;
            right_peak = right_peak.max(right.abs());
        }
    }

    if channels == 1 {
        right_peak = left_peak;
    }

    (left_peak.clamp(0.0, 1.0), right_peak.clamp(0.0, 1.0))
}

fn smooth_level(previous: f32, input: f32) -> f32 {
    let delta = input - previous;

    if delta > 0.0 {
        previous + delta * 0.28
    } else {
        previous + delta * 0.08
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
