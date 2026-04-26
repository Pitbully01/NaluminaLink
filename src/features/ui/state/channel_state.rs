use std::collections::BTreeMap;

pub const DEFAULT_CHANNEL_LEVEL: f32 = 1.0;
pub const DEFAULT_MONITOR_SEND: f32 = 1.0;
pub const DEFAULT_STREAM_SEND: f32 = 0.82;
pub const MAX_VISIBLE_CHANNELS: usize = 10;
pub const DEFAULT_MIX_BUS_COUNT: usize = 2;
pub const MAX_MIX_BUS_COUNT: usize = 8;

#[derive(Clone, Debug)]
pub struct ChannelStripState {
    pub level: f32,
    pub muted: bool,
    pub sends: Vec<f32>,
}

impl ChannelStripState {
    pub fn meter_value(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.level
        }
    }

    fn normalized_with(&self, defaults: &ChannelStripState) -> ChannelStripState {
        let mut sends = self.sends.clone();
        let target_len = defaults.sends.len();

        if sends.len() < target_len {
            sends.extend(defaults.sends[sends.len()..target_len].iter().copied());
        }

        if sends.len() > target_len {
            sends.truncate(target_len);
        }

        ChannelStripState {
            level: self.level,
            muted: self.muted,
            sends,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MixLevels {
    pub buses: Vec<f32>,
}

pub(in crate::features::ui) struct ChannelStateStore {
    channels: BTreeMap<u32, ChannelStripState>,
}

impl ChannelStateStore {
    pub(in crate::features::ui) fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
        }
    }

    pub(in crate::features::ui) fn ensure_defaults(
        &mut self,
        node_id: u32,
        defaults: ChannelStripState,
    ) {
        self.channels
            .entry(node_id)
            .and_modify(|state| *state = state.normalized_with(&defaults))
            .or_insert(defaults);
    }

    pub(in crate::features::ui) fn load_or_default(
        &mut self,
        node_id: u32,
        defaults: ChannelStripState,
    ) -> ChannelStripState {
        self.ensure_defaults(node_id, defaults);

        self.channels
            .get(&node_id)
            .cloned()
            .unwrap_or(ChannelStripState {
                level: DEFAULT_CHANNEL_LEVEL,
                muted: false,
                sends: Vec::new(),
            })
    }

    pub(in crate::features::ui) fn store(&mut self, node_id: u32, state: ChannelStripState) {
        self.channels.insert(node_id, state);
    }

    pub(in crate::features::ui) fn effective_mix(&self, node_id: u32, bus_index: usize) -> f32 {
        let Some(state) = self.channels.get(&node_id) else {
            return 0.0;
        };

        let send = state.sends.get(bus_index).copied().unwrap_or(0.0);

        if state.muted {
            0.0
        } else {
            state.level * send
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChannelStateStore;
    use super::ChannelStripState;

    fn defaults() -> ChannelStripState {
        ChannelStripState {
            level: 0.5,
            muted: false,
            sends: vec![0.8, 0.6],
        }
    }

    #[test]
    fn load_or_default_initializes_missing_node_state() {
        let mut store = ChannelStateStore::new();
        let state = store.load_or_default(10, defaults());

        assert_eq!(state.level, 0.5);
        assert_eq!(state.sends[0], 0.8);
        assert_eq!(state.sends[1], 0.6);
        assert!(!state.muted);
    }

    #[test]
    fn store_overrides_previous_values() {
        let mut store = ChannelStateStore::new();
        let mut state = store.load_or_default(77, defaults());
        state.level = 0.9;
        state.sends[0] = 0.2;
        state.sends[1] = 0.3;
        state.muted = true;
        store.store(77, state);

        let reloaded = store.load_or_default(77, defaults());
        assert_eq!(reloaded.level, 0.9);
        assert_eq!(reloaded.sends[0], 0.2);
        assert_eq!(reloaded.sends[1], 0.3);
        assert!(reloaded.muted);
    }

    #[test]
    fn effective_mix_respects_selected_bus_and_mute() {
        let mut store = ChannelStateStore::new();
        let state = ChannelStripState {
            level: 0.75,
            muted: false,
            sends: vec![0.4, 0.9],
        };
        store.store(5, state.clone());

        let monitor_mix = store.effective_mix(5, 0);
        let stream_mix = store.effective_mix(5, 1);
        assert!((monitor_mix - 0.3).abs() < f32::EPSILON);
        assert!((stream_mix - 0.675).abs() < f32::EPSILON);

        store.store(
            5,
            ChannelStripState {
                muted: true,
                ..state
            },
        );
        assert_eq!(store.effective_mix(5, 0), 0.0);
        assert_eq!(store.effective_mix(5, 1), 0.0);
    }

    #[test]
    fn load_or_default_resizes_sends_when_bus_count_changes() {
        let mut store = ChannelStateStore::new();

        store.store(
            9,
            ChannelStripState {
                level: 0.5,
                muted: false,
                sends: vec![0.7, 0.4],
            },
        );

        let expanded = store.load_or_default(
            9,
            ChannelStripState {
                level: 0.5,
                muted: false,
                sends: vec![1.0, 0.8, 0.6],
            },
        );

        assert_eq!(expanded.sends, vec![0.7, 0.4, 0.6]);

        let collapsed = store.load_or_default(
            9,
            ChannelStripState {
                level: 0.5,
                muted: false,
                sends: vec![1.0],
            },
        );

        assert_eq!(collapsed.sends, vec![0.7]);
    }
}
