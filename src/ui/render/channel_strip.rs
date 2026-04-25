use eframe::egui;

use super::super::components::percent_progress_bar;
use super::super::NaluminaApp;
use crate::models::{ChannelStripState, NodeEntry};

impl NaluminaApp {
    fn channel_label(name: &str) -> String {
        let mut chars = name.chars();
        let short_name: String = chars.by_ref().take(14).collect();
        if chars.next().is_some() {
            format!("{}...", short_name)
        } else {
            short_name
        }
    }

    fn load_channel_state(&mut self, node_id: u32) -> ChannelStripState {
        self.channel_state
            .load_or_default(node_id, Self::default_channel_state())
    }

    fn store_channel_state(&mut self, node_id: u32, state: ChannelStripState) {
        self.channel_state.store(node_id, state);
    }

    fn draw_meter_and_routes(&mut self, ui: &mut egui::Ui, state: &mut ChannelStripState) {
        let meter_value = state.meter_value();
        ui.add_space(8.0);
        percent_progress_bar(ui, meter_value, 60.0, egui::Color32::from_rgb(0, 197, 143));

        if ui
            .add_sized(
                [52.0, 26.0],
                egui::Button::new(if state.muted {
                    self.i18n.text("ui.channel.unmute")
                } else {
                    self.i18n.text("ui.channel.mute")
                }),
            )
            .clicked()
        {
            state.muted = !state.muted;
        }

        ui.add_space(6.0);
        ui.label(egui::RichText::new(self.i18n.text("ui.channel.monitor_send")).small());
        ui.add_sized(
            [60.0, 16.0],
            egui::Slider::new(&mut state.send_monitor, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );

        ui.label(egui::RichText::new(self.i18n.text("ui.channel.stream_send")).small());
        ui.add_sized(
            [60.0, 16.0],
            egui::Slider::new(&mut state.send_stream, 0.0..=1.0)
                .show_value(false)
                .trailing_fill(true),
        );
    }

    pub(super) fn draw_channel_strip(&mut self, ui: &mut egui::Ui, node: &NodeEntry) {
        let mut state = self.load_channel_state(node.id);
        let label = Self::channel_label(&node.name);

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 28, 39))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 62, 88)))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.set_width(128.0);
                ui.vertical_centered(|ui| {
                    ui.strong(label);
                    ui.label(
                        egui::RichText::new(
                            self.i18n
                                .text_with("ui.channel.id", &[("id", node.id.to_string())]),
                        )
                        .size(11.0),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let slider = egui::Slider::new(&mut state.level, 0.0..=1.0)
                            .vertical()
                            .show_value(false)
                            .trailing_fill(true);
                        ui.add_sized([24.0, 150.0], slider);

                        ui.vertical(|ui| {
                            self.draw_meter_and_routes(ui, &mut state);
                        });
                    });
                });
            });

        self.store_channel_state(node.id, state);
    }
}
