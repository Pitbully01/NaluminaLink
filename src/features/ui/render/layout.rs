use eframe::egui;
use log::info;

use super::super::components::{percent_progress_bar, section_header};
use super::super::NaluminaApp;
use crate::features::node_discovery::NodeEntry;
use crate::features::ui::state::{MixLevels, MAX_MIX_BUS_COUNT};

const SCENE_PRESET_BALANCED: usize = 0;
const SCENE_PRESET_MONITOR_FOCUS: usize = 1;
const SCENE_PRESET_STREAM_BOOST: usize = 2;
const MAX_VISIBLE_CHANNEL_LIMIT: u32 = 24;

impl NaluminaApp {
    pub(super) fn mix_bus_label(&self, bus_index: usize) -> String {
        match bus_index {
            0 => self.i18n.text("ui.channel.monitor_send"),
            1 => self.i18n.text("ui.channel.stream_send"),
            2 => self.i18n.text("ui.bus.chat"),
            3 => self.i18n.text("ui.bus.fx_return"),
            _ => format!("BUS {}", bus_index + 1),
        }
    }

    fn scene_preset_name(&self, preset: usize) -> String {
        match preset {
            SCENE_PRESET_MONITOR_FOCUS => self.i18n.text("ui.preset.monitor_focus"),
            SCENE_PRESET_STREAM_BOOST => self.i18n.text("ui.preset.stream_boost"),
            _ => self.i18n.text("ui.preset.balanced"),
        }
    }

    fn apply_scene_preset(&mut self, preset: usize) {
        let node_ids: Vec<u32> = self.visible_nodes().iter().map(|node| node.id).collect();
        for node_id in node_ids {
            let mut state = self
                .channel_state
                .load_or_default(node_id, Self::default_channel_state(self.mix_bus_count));

            match preset {
                SCENE_PRESET_MONITOR_FOCUS => {
                    if let Some(send) = state.sends.get_mut(0) {
                        *send = 1.0;
                    }
                    if let Some(send) = state.sends.get_mut(1) {
                        *send = 0.55;
                    }
                }
                SCENE_PRESET_STREAM_BOOST => {
                    if let Some(send) = state.sends.get_mut(0) {
                        *send = 0.7;
                    }
                    if let Some(send) = state.sends.get_mut(1) {
                        *send = 1.0;
                    }
                }
                _ => {
                    for send in &mut state.sends {
                        *send = 0.9;
                    }
                }
            }

            self.channel_state.store(node_id, state);
        }

        self.selected_scene_preset = preset;
        let preset_name = self.scene_preset_name(preset);
        self.status.set_scene_applied(&self.i18n, preset_name);
        info!(
            "scene: applied preset {} to {} visible channels",
            preset,
            self.visible_nodes().len()
        );
    }

    fn render_scene_preset_buttons(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new(self.i18n.text("ui.label.scene_presets")).strong());

        let presets = [
            SCENE_PRESET_BALANCED,
            SCENE_PRESET_MONITOR_FOCUS,
            SCENE_PRESET_STREAM_BOOST,
        ];

        for preset in presets {
            let selected = self.selected_scene_preset == preset;
            let button = egui::Button::new(self.scene_preset_name(preset)).fill(if selected {
                egui::Color32::from_rgb(0, 114, 204)
            } else {
                egui::Color32::from_rgb(33, 44, 62)
            });

            if ui.add(button).clicked() {
                self.apply_scene_preset(preset);
            }
        }
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
                    }

                    ui.separator();
                    self.render_scene_preset_buttons(ui);
                });
            });
    }

    fn visible_nodes(&self) -> Vec<NodeEntry> {
        let filter = self.node_filter.trim().to_lowercase();

        self.nodes
            .iter()
            .filter(|node| {
                if filter.is_empty() {
                    return true;
                }

                let id_match = node.id.to_string().contains(&filter);
                let name_match = node.name.to_lowercase().contains(&filter);
                let description_match = node.description.to_lowercase().contains(&filter);

                id_match || name_match || description_match
            })
            .take(self.visible_channel_limit)
            .cloned()
            .collect()
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

                    let visible_nodes = self.visible_nodes();

                    if visible_nodes.is_empty() {
                        ui.label(self.i18n.text("ui.nodes.filter_empty"));
                        return;
                    }

                    for node in &visible_nodes {
                        self.draw_channel_strip(ui, node);
                    }
                });
            });
    }

    fn render_mix_outputs(&self, ui: &mut egui::Ui, mix_levels: &MixLevels) {
        section_header(
            ui,
            self.i18n.text("ui.section.mix_outputs"),
            self.i18n.text("ui.section.mix_outputs_subtitle"),
        );
        ui.add_space(8.0);

        let palette = [
            egui::Color32::from_rgb(0, 168, 255),
            egui::Color32::from_rgb(0, 197, 143),
            egui::Color32::from_rgb(231, 177, 34),
            egui::Color32::from_rgb(255, 112, 67),
            egui::Color32::from_rgb(125, 187, 255),
            egui::Color32::from_rgb(135, 216, 176),
            egui::Color32::from_rgb(255, 205, 105),
            egui::Color32::from_rgb(255, 151, 127),
        ];

        for (bus_index, level) in mix_levels.buses.iter().enumerate() {
            if bus_index > 0 {
                ui.add_space(6.0);
            }

            ui.label(egui::RichText::new(self.mix_bus_label(bus_index)).strong());
            percent_progress_bar(ui, *level, 220.0, palette[bus_index % palette.len()]);
        }
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
                let visible_nodes = self.visible_nodes();
                for node in &visible_nodes {
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

                ui.label(
                    egui::RichText::new(self.i18n.text_with(
                        "ui.nodes.visible_count",
                        &[
                            ("shown", visible_nodes.len().to_string()),
                            ("total", self.nodes.len().to_string()),
                        ],
                    ))
                    .small(),
                );
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
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_workspace_controls(ui);
            ui.add_space(10.0);
            self.render_channel_rack(ui);
            ui.add_space(14.0);

            let mix_levels = self.calculate_mix_levels();
            ui.columns(2, |columns| {
                self.render_mix_outputs(&mut columns[0], &mix_levels);
                self.render_node_browser(&mut columns[1]);
            });

            self.render_scene_summary(ui, &mix_levels);
        });
    }
}
