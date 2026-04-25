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
