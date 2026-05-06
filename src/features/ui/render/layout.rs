use eframe::egui;
use log::debug;

use super::super::components::section_header;
use super::super::{InputChannel, NaluminaApp};
use super::cards;
use crate::features::ui::state::{MixLevels, MAX_MIX_BUS_COUNT};

const MAX_VISIBLE_CHANNEL_LIMIT: u32 = 24;

impl NaluminaApp {
    fn sync_mix_bus_names(&mut self) {
        let target = self.mix_bus_count;

        if self.mix_bus_names.len() < target {
            let start = self.mix_bus_names.len();
            for bus_index in start..target {
                self.mix_bus_names
                    .push(Self::default_mix_bus_name(&self.i18n, bus_index));
            }
        }

        if self.mix_bus_names.len() > target {
            self.mix_bus_names.truncate(target);
        }
    }

    fn add_input_channel(&mut self) {
        let id = self.next_input_channel_id;
        self.next_input_channel_id = self.next_input_channel_id.saturating_add(1);

        self.input_channels.push(InputChannel {
            id,
            name: self
                .i18n
                .text_with("ui.input.default_name", &[("index", id.to_string())]),
            source_node_id: None,
        });

        self.ensure_input_channel_defaults(id, None);
    }

    pub(in crate::features::ui) fn mix_bus_label(&self, bus_index: usize) -> String {
        self.mix_bus_names
            .get(bus_index)
            .cloned()
            .unwrap_or_else(|| Self::default_mix_bus_name(&self.i18n, bus_index))
    }

    fn render_workspace_controls(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(20, 26, 38))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 65, 92)))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                section_header(
                    ui,
                    self.i18n.text("ui.section.workspace_controls"),
                    self.i18n.text("ui.section.workspace_controls_subtitle"),
                );
                ui.add_space(6.0);

                ui.horizontal_wrapped(|ui| {
                    ui.label(self.i18n.text("ui.label.node_filter"));
                    ui.add_sized(
                        [220.0, 24.0],
                        egui::TextEdit::singleline(&mut self.node_filter)
                            .hint_text(self.i18n.text("ui.placeholder.node_filter")),
                    );

                    ui.separator();
                    ui.label(self.i18n.text("ui.label.visible_channels"));
                    let mut visible_limit = self.visible_channel_limit as u32;
                    if ui
                        .add(
                            egui::DragValue::new(&mut visible_limit)
                                .range(1..=MAX_VISIBLE_CHANNEL_LIMIT)
                                .speed(0.25),
                        )
                        .changed()
                    {
                        self.visible_channel_limit = visible_limit as usize;
                    }

                    ui.separator();
                    ui.label(self.i18n.text("ui.label.mix_outputs_count"));
                    let mut mix_outputs = self.mix_bus_count as u32;
                    if ui
                        .add(
                            egui::DragValue::new(&mut mix_outputs)
                                .range(1..=MAX_MIX_BUS_COUNT as u32)
                                .speed(0.2),
                        )
                        .changed()
                    {
                        self.mix_bus_count = mix_outputs as usize;
                        self.sync_mix_bus_names();
                        self.sync_input_channel_defaults();
                    }

                    ui.separator();
                    if ui
                        .button(self.i18n.text("ui.button.add_input"))
                        .on_hover_text(self.i18n.text("ui.button.add_input_hint"))
                        .clicked()
                    {
                        self.add_input_channel();
                    }
                });
            });
    }

    pub(in crate::features::ui) fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(14, 20, 31))
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(self.i18n.text("app.window_title"));
                        ui.label(self.i18n.text("ui.tagline"));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(self.i18n.text("ui.button.refresh_nodes"))
                                        .fill(egui::Color32::from_rgb(0, 114, 204)),
                                )
                                .clicked()
                            {
                                self.start_refresh();
                            }

                            if ui.button(self.i18n.text("ui.button.doctor")).clicked() {
                                self.status.set_doctor_message(&self.i18n);
                            }
                        });
                    });
                });
        });
    }

    pub(in crate::features::ui) fn render_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.i18n.text("ui.label.status")).strong());
                    ui.label(self.status.text());
                });
            });
    }

    fn render_scene_summary(&self, ui: &mut egui::Ui, mix_levels: &MixLevels) {
        let monitor = mix_levels.buses.first().copied().unwrap_or(0.0);
        let stream = mix_levels.buses.get(1).copied().unwrap_or(0.0);

        ui.add_space(10.0);
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(17, 23, 35))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 72, 98)))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(self.i18n.text("ui.scene")).strong());
                    ui.label(self.i18n.text("ui.scene.default_streaming"));
                    ui.separator();
                    ui.label(egui::RichText::new(self.i18n.text("ui.output")).strong());
                    ui.label(self.i18n.text_with(
                        "ui.route.summary",
                        &[
                            ("monitor", format!("{:.0}%", monitor * 100.0)),
                            ("stream", format!("{:.0}%", stream * 100.0)),
                        ],
                    ));
                });
            });
    }

    pub(in crate::features::ui) fn render_main_panel(&mut self, ctx: &egui::Context) {
        debug!(
            "ui:render_main_panel nodes={} input_channels={} mix_bus_count={} visible_limit={}",
            self.nodes.len(),
            self.input_channels.len(),
            self.mix_bus_count,
            self.visible_channel_limit
        );
        egui::CentralPanel::default().show(ctx, |ui| {
            let node_choices = self.node_choices();
            debug!("ui:render_main_panel node_choices={}", node_choices.len());
            self.sync_mix_bus_names();
            self.sync_input_channel_defaults();
            self.render_workspace_controls(ui);
            ui.add_space(10.0);
            cards::render_mix_matrix(self, ui, &node_choices);
            ui.add_space(14.0);

            let mix_levels = self.calculate_mix_levels();
            ui.columns(2, |columns| {
                cards::render_mix_outputs(self, &mut columns[0], &mix_levels);
                self.render_node_browser(&mut columns[1]);
            });

            self.render_scene_summary(ui, &mix_levels);
        });
    }
}
