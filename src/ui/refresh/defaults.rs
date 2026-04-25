use super::super::NaluminaApp;
use crate::models::{
    ChannelStripState, DEFAULT_CHANNEL_LEVEL, DEFAULT_MONITOR_SEND, DEFAULT_STREAM_SEND,
};

impl NaluminaApp {
    pub(in crate::ui) fn default_channel_state() -> ChannelStripState {
        ChannelStripState {
            level: DEFAULT_CHANNEL_LEVEL,
            muted: false,
            send_monitor: DEFAULT_MONITOR_SEND,
            send_stream: DEFAULT_STREAM_SEND,
        }
    }

    pub(in crate::ui) fn ensure_node_defaults(&mut self, node_id: u32) {
        let state = Self::default_channel_state();
        self.channel_state.ensure_defaults(node_id, state);
    }

    pub(in crate::ui::refresh) fn sync_node_defaults(&mut self) {
        let node_ids: Vec<u32> = self.nodes.iter().map(|node| node.id).collect();
        for node_id in node_ids {
            self.ensure_node_defaults(node_id);
        }
    }
}
