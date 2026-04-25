use eframe::egui;

use super::super::NaluminaApp;
use super::super::components::{percent_progress_bar, section_header};
use crate::models::{MixLevels, NodeEntry, MAX_VISIBLE_CHANNELS};

impl NaluminaApp {
    pub(in crate::ui) fn render_top_bar(&mut self, ctx: &egui::Context) {
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

    pub(in crate::ui) fn render_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.i18n.text("ui.label.status")).strong());
                    ui.label(self.status.text());
                });
            });
    }

    fn render_channel_rack(&mut self, ui: &mut egui::Ui) {
        section_header(
            ui,
            self.i18n.text("ui.section.channel_rack"),
            self.i18n.text("ui.section.channel_rack_subtitle"),
        );
        ui.add_space(6.0);

        egui::ScrollArea::horizontal()
            .id_source("channel_rack")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.nodes.is_empty() {
                        ui.label(self.i18n.text("nodes.empty"));
                        return;
                    }

                    let visible_nodes: Vec<NodeEntry> = self
                        .nodes
                        .iter()
                        .take(MAX_VISIBLE_CHANNELS)
                        .cloned()
                        .collect();

                    for node in &visible_nodes {
                        self.draw_channel_strip(ui, node);
                    }
                });
            });
    }

    fn render_mix_outputs(&self, ui: &mut egui::Ui, mix_levels: MixLevels) {
        section_header(
            ui,
            self.i18n.text("ui.section.mix_outputs"),
            self.i18n.text("ui.section.mix_outputs_subtitle"),
        );
        ui.add_space(8.0);

        ui.label(egui::RichText::new(self.i18n.text("ui.label.monitor_mix_level")).strong());
        percent_progress_bar(
            ui,
            mix_levels.monitor,
            220.0,
            egui::Color32::from_rgb(0, 168, 255),
        );

        ui.add_space(6.0);
        ui.label(egui::RichText::new(self.i18n.text("ui.label.stream_mix_level")).strong());
        percent_progress_bar(
            ui,
            mix_levels.stream,
            220.0,
            egui::Color32::from_rgb(0, 197, 143),
        );
    }

    fn render_node_browser(&self, ui: &mut egui::Ui) {
        section_header(
            ui,
            self.i18n.text("ui.section.node_browser"),
            self.i18n.text("ui.section.node_browser_subtitle"),
        );

        egui::ScrollArea::vertical()
            .id_source("node_browser")
            .max_height(210.0)
            .show(ui, |ui| {
                for node in &self.nodes {
                    ui.horizontal_wrapped(|ui| {
                        ui.strong(format!("#{}", node.id));
                        ui.label(&node.name);
                        if !node.description.is_empty() {
                            ui.label(
                                egui::RichText::new(self.i18n.text_with(
                                    "ui.node.description_format",
                                    &[("description", node.description.clone())],
                                ))
                                .small(),
                            );
                        }
                    });
                    ui.separator();
                }
            });
    }

    fn render_scene_summary(&self, ui: &mut egui::Ui, mix_levels: MixLevels) {
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
                            ("monitor", format!("{:.0}%", mix_levels.monitor * 100.0)),
                            ("stream", format!("{:.0}%", mix_levels.stream * 100.0)),
                        ],
                    ));
                });
            });
    }

    pub(in crate::ui) fn render_main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_channel_rack(ui);
            ui.add_space(14.0);

            let mix_levels = self.calculate_mix_levels();
            ui.columns(2, |columns| {
                self.render_mix_outputs(&mut columns[0], mix_levels);
                self.render_node_browser(&mut columns[1]);
            });

            self.render_scene_summary(ui, mix_levels);
        });
    }
}
