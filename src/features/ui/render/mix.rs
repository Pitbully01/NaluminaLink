use super::super::state::MixBus;
use super::super::NaluminaApp;
use crate::features::ui::state::MixLevels;

impl NaluminaApp {
    fn calculate_mix_level(&self, bus: MixBus) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }

        let sum = self
            .nodes
            .iter()
            .map(|node| self.channel_state.effective_mix(node.id, bus))
            .sum::<f32>();

        sum / self.nodes.len() as f32
    }

    pub(super) fn calculate_mix_levels(&self) -> MixLevels {
        MixLevels {
            monitor: self.calculate_mix_level(MixBus::Monitor),
            stream: self.calculate_mix_level(MixBus::Stream),
        }
    }
}
