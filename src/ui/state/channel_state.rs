use std::collections::BTreeMap;

use crate::models::ChannelStripState;

#[derive(Clone, Copy, Debug)]
pub(in crate::ui) enum MixBus {
    Monitor,
    Stream,
}

pub(in crate::ui) struct ChannelStateStore {
    channel_levels: BTreeMap<u32, f32>,
    channel_mute: BTreeMap<u32, bool>,
    channel_send_monitor: BTreeMap<u32, f32>,
    channel_send_stream: BTreeMap<u32, f32>,
}

impl ChannelStateStore {
    pub(in crate::ui) fn new() -> Self {
        Self {
            channel_levels: BTreeMap::new(),
            channel_mute: BTreeMap::new(),
            channel_send_monitor: BTreeMap::new(),
            channel_send_stream: BTreeMap::new(),
        }
    }

    pub(in crate::ui) fn ensure_defaults(&mut self, node_id: u32, defaults: ChannelStripState) {
        self.channel_levels.entry(node_id).or_insert(defaults.level);
        self.channel_mute.entry(node_id).or_insert(defaults.muted);
        self.channel_send_monitor
            .entry(node_id)
            .or_insert(defaults.send_monitor);
        self.channel_send_stream
            .entry(node_id)
            .or_insert(defaults.send_stream);
    }

    pub(in crate::ui) fn load_or_default(
        &mut self,
        node_id: u32,
        defaults: ChannelStripState,
    ) -> ChannelStripState {
        self.ensure_defaults(node_id, defaults);

        ChannelStripState {
            level: *self.channel_levels.get(&node_id).unwrap_or(&defaults.level),
            muted: *self.channel_mute.get(&node_id).unwrap_or(&defaults.muted),
            send_monitor: *self
                .channel_send_monitor
                .get(&node_id)
                .unwrap_or(&defaults.send_monitor),
            send_stream: *self
                .channel_send_stream
                .get(&node_id)
                .unwrap_or(&defaults.send_stream),
        }
    }

    pub(in crate::ui) fn store(&mut self, node_id: u32, state: ChannelStripState) {
        self.channel_levels.insert(node_id, state.level);
        self.channel_mute.insert(node_id, state.muted);
        self.channel_send_monitor
            .insert(node_id, state.send_monitor);
        self.channel_send_stream.insert(node_id, state.send_stream);
    }

    pub(in crate::ui) fn effective_mix(&self, node_id: u32, bus: MixBus) -> f32 {
        let level = *self.channel_levels.get(&node_id).unwrap_or(&0.0);
        let muted = *self.channel_mute.get(&node_id).unwrap_or(&false);
        let send = match bus {
            MixBus::Monitor => *self.channel_send_monitor.get(&node_id).unwrap_or(&0.0),
            MixBus::Stream => *self.channel_send_stream.get(&node_id).unwrap_or(&0.0),
        };

        if muted {
            0.0
        } else {
            level * send
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChannelStateStore, MixBus};
    use crate::models::ChannelStripState;

    fn defaults() -> ChannelStripState {
        ChannelStripState {
            level: 0.5,
            muted: false,
            send_monitor: 0.8,
            send_stream: 0.6,
        }
    }

    #[test]
    fn load_or_default_initializes_missing_node_state() {
        let mut store = ChannelStateStore::new();
        let state = store.load_or_default(10, defaults());

        assert_eq!(state.level, 0.5);
        assert_eq!(state.send_monitor, 0.8);
        assert_eq!(state.send_stream, 0.6);
        assert!(!state.muted);
    }

    #[test]
    fn store_overrides_previous_values() {
        let mut store = ChannelStateStore::new();
        let mut state = store.load_or_default(77, defaults());
        state.level = 0.9;
        state.send_monitor = 0.2;
        state.send_stream = 0.3;
        state.muted = true;
        store.store(77, state);

        let reloaded = store.load_or_default(77, defaults());
        assert_eq!(reloaded.level, 0.9);
        assert_eq!(reloaded.send_monitor, 0.2);
        assert_eq!(reloaded.send_stream, 0.3);
        assert!(reloaded.muted);
    }

    #[test]
    fn effective_mix_respects_selected_bus_and_mute() {
        let mut store = ChannelStateStore::new();
        let state = ChannelStripState {
            level: 0.75,
            muted: false,
            send_monitor: 0.4,
            send_stream: 0.9,
        };
        store.store(5, state);

        let monitor_mix = store.effective_mix(5, MixBus::Monitor);
        let stream_mix = store.effective_mix(5, MixBus::Stream);
        assert!((monitor_mix - 0.3).abs() < f32::EPSILON);
        assert!((stream_mix - 0.675).abs() < f32::EPSILON);

        store.store(
            5,
            ChannelStripState {
                muted: true,
                ..state
            },
        );
        assert_eq!(store.effective_mix(5, MixBus::Monitor), 0.0);
        assert_eq!(store.effective_mix(5, MixBus::Stream), 0.0);
    }
}
