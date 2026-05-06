use super::super::NaluminaApp;
use crate::features::node_discovery::find_default_microphone;
use crate::features::ui::state::{
    ChannelStripState, DEFAULT_CHANNEL_LEVEL, DEFAULT_MONITOR_SEND, DEFAULT_STREAM_SEND,
};
use log::debug;

impl NaluminaApp {
    fn clamped_level(level: Option<f32>) -> f32 {
        level.unwrap_or(DEFAULT_CHANNEL_LEVEL).clamp(0.0, 1.0)
    }

    fn default_sends(mix_bus_count: usize) -> Vec<f32> {
        let mut sends = vec![0.9; mix_bus_count];

        if !sends.is_empty() {
            sends[0] = DEFAULT_MONITOR_SEND;
        }

        if sends.len() > 1 {
            sends[1] = DEFAULT_STREAM_SEND;
        }

        sends
    }

    fn default_output_mutes(mix_bus_count: usize) -> Vec<bool> {
        vec![false; mix_bus_count]
    }

    pub(in crate::features::ui) fn source_volume_hint(
        &self,
        source_node_id: Option<u32>,
    ) -> Option<f32> {
        let node_id = source_node_id?;
        self.nodes
            .iter()
            .find(|node| node.id == node_id)
            .and_then(|node| node.volume_hint)
    }

    pub(in crate::features::ui) fn default_channel_state(
        mix_bus_count: usize,
        level_hint: Option<f32>,
    ) -> ChannelStripState {
        ChannelStripState {
            level: Self::clamped_level(level_hint),
            muted: false,
            sends: Self::default_sends(mix_bus_count),
            output_mutes: Self::default_output_mutes(mix_bus_count),
        }
    }

    pub(in crate::features::ui) fn ensure_input_channel_defaults(
        &mut self,
        channel_id: u32,
        source_node_id: Option<u32>,
    ) {
        let volume_hint = self.source_volume_hint(source_node_id);
        let state = Self::default_channel_state(self.mix_bus_count, volume_hint);
        let changed = self.channel_state.ensure_defaults(channel_id, state);

        debug!(
            "ensure_input_channel_defaults: channel_id={} source_node_id={:?} changed={}",
            channel_id, source_node_id, changed
        );
    }

    pub(in crate::features::ui) fn sync_input_channel_defaults(&mut self) {
        let channels: Vec<(u32, Option<u32>)> = self
            .input_channels
            .iter()
            .map(|channel| (channel.id, channel.source_node_id))
            .collect();

        debug!(
            "sync_input_channel_defaults: channels={} nodes={} mix_bus_count={}",
            channels.len(),
            self.nodes.len(),
            self.mix_bus_count
        );

        for (channel_id, source_node_id) in channels {
            debug!(
                "sync_input_channel_defaults: channel_id={} source_node_id={:?}",
                channel_id, source_node_id
            );
            self.ensure_input_channel_defaults(channel_id, source_node_id);
        }
    }

    pub(in crate::features::ui) fn apply_default_devices(&mut self) {
        debug!(
            "apply_default_devices: checking {} input channels",
            self.input_channels.len()
        );

        debug!(
            "apply_default_devices: nodes available = {:?}",
            self.nodes
                .iter()
                .map(|node| (&node.id, &node.name, &node.media_class, &node.device_class))
                .collect::<Vec<_>>()
        );

        if let Some(mic_id) = find_default_microphone(&self.nodes) {
            debug!("apply_default_devices: found microphone with id {}", mic_id);
            if let Some(channel) = self
                .input_channels
                .iter_mut()
                .find(|ch| ch.source_node_id.is_none())
            {
                debug!(
                    "apply_default_devices: assigning microphone {} to channel {}",
                    mic_id, channel.id
                );
                channel.source_node_id = Some(mic_id);
            } else {
                debug!("apply_default_devices: no unassigned channels found");
            }
        } else {
            debug!("apply_default_devices: no microphone found");
        }
    }
}
