pub const DEFAULT_CHANNEL_LEVEL: f32 = 0.72;
pub const DEFAULT_MONITOR_SEND: f32 = 1.0;
pub const DEFAULT_STREAM_SEND: f32 = 0.82;
pub const MAX_VISIBLE_CHANNELS: usize = 10;

#[derive(Clone, Debug)]
pub struct NodeEntry {
    pub id: u32,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Copy, Debug)]
pub struct ChannelStripState {
    pub level: f32,
    pub muted: bool,
    pub send_monitor: f32,
    pub send_stream: f32,
}

impl ChannelStripState {
    pub fn meter_value(self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.level
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MixLevels {
    pub monitor: f32,
    pub stream: f32,
}
