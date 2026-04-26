use super::super::NaluminaApp;
use crate::features::ui::state::MixLevels;

impl NaluminaApp {
    fn calculate_mix_level(&self, bus_index: usize) -> f32 {
        if self.input_channels.is_empty() {
            return 0.0;
        }

        let sum = self
            .input_channels
            .iter()
            .map(|channel| self.channel_state.effective_mix(channel.id, bus_index))
            .sum::<f32>();

        sum / self.input_channels.len() as f32
    }

    pub(super) fn calculate_mix_levels(&self) -> MixLevels {
        let buses = (0..self.mix_bus_count)
            .map(|bus_index| self.calculate_mix_level(bus_index))
            .collect::<Vec<f32>>();

        MixLevels { buses }
    }
}
