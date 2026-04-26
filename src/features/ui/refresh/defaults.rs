use super::super::NaluminaApp;
use crate::features::node_discovery::NodeEntry;
use crate::features::ui::state::{
    ChannelStripState, DEFAULT_CHANNEL_LEVEL, DEFAULT_MONITOR_SEND, DEFAULT_STREAM_SEND,
};

impl NaluminaApp {
    fn clamped_level(level: Option<f32>) -> f32 {
        level.unwrap_or(DEFAULT_CHANNEL_LEVEL).clamp(0.0, 1.0)
    }

    pub(in crate::features::ui) fn default_channel_state() -> ChannelStripState {
        ChannelStripState {
            level: DEFAULT_CHANNEL_LEVEL,
            muted: false,
            send_monitor: DEFAULT_MONITOR_SEND,
            send_stream: DEFAULT_STREAM_SEND,
        }
    }

    pub(in crate::features::ui) fn ensure_node_defaults(&mut self, node: &NodeEntry) {
        let mut state = Self::default_channel_state();
        state.level = Self::clamped_level(node.volume_hint);

        self.channel_state.ensure_defaults(node.id, state);
    }

    pub(in crate::features::ui::refresh) fn sync_node_defaults(&mut self) {
        let nodes: Vec<NodeEntry> = self.nodes.clone();
        for node in &nodes {
            self.ensure_node_defaults(node);
        }
    }
}
