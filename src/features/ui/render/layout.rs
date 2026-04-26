use eframe::egui;

use super::super::components::{percent_progress_bar, section_header};
use super::super::{InputChannel, NaluminaApp};
use crate::features::ui::state::{MixLevels, MAX_MIX_BUS_COUNT};

const MAX_VISIBLE_CHANNEL_LIMIT: u32 = 24;
const FADER_DB_MIN: f32 = -60.0;
const FADER_DB_MAX: f32 = 12.0;
const MUTE_DB_EPSILON: f32 = 0.001;

impl NaluminaApp {
    fn gain_to_db(gain: f32) -> f32 {
        if gain <= MUTE_DB_EPSILON {
            FADER_DB_MIN
        } else {
            (20.0 * gain.log10()).clamp(FADER_DB_MIN, FADER_DB_MAX)
        }
    }

    fn db_to_gain(db: f32) -> f32 {
        if db <= FADER_DB_MIN + 0.05 {
            0.0
        } else {
            10f32.powf(db / 20.0)
        }
    }

    fn format_db(db: f32) -> String {
        if db <= FADER_DB_MIN + 0.05 {
            "-inf dB".to_string()
        } else {
            format!("{db:.1} dB")
        }
    }

    fn meter_zone_color(level: f32) -> egui::Color32 {
        if level < 0.65 {
            egui::Color32::from_rgb(0, 197, 143)
        } else if level < 0.85 {
            egui::Color32::from_rgb(231, 177, 34)
        } else {
            egui::Color32::from_rgb(219, 68, 55)
        }
    }

    fn render_zone_meter(ui: &mut egui::Ui, level: f32, width: f32) {
        let clamped = level.clamp(0.0, 1.0);
        ui.add(
            egui::ProgressBar::new(clamped)
                .desired_width(width)
                .fill(Self::meter_zone_color(clamped))
                .show_percentage(),
        );
    }

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

    fn source_label(&self, source_node_id: Option<u32>) -> String {
        let Some(node_id) = source_node_id else {
            return self.i18n.text("ui.device.unassigned");
        };

        self.nodes
            .iter()
            .find(|node| node.id == node_id)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| self.i18n.text("ui.device.unassigned"))
    }

    fn source_layout_label(&self, source_node_id: Option<u32>) -> String {
        let channels = source_node_id
            .and_then(|id| self.nodes.iter().find(|node| node.id == id))
            .and_then(|node| node.channels_hint)
            .unwrap_or(2);

        if channels <= 1 {
            self.i18n.text("ui.layout.mono")
        } else {
            self.i18n.text("ui.layout.stereo")
        }
    }

    fn visible_input_channels(&self) -> Vec<InputChannel> {
        let filter = self.node_filter.trim().to_lowercase();

        self.input_channels
            .iter()
            .filter(|channel| {
                if filter.is_empty() {
                    return true;
                }

                let id_match = channel.id.to_string().contains(&filter);
                let name_match = channel.name.to_lowercase().contains(&filter);
                let source_match = self
                    .source_label(channel.source_node_id)
                    .to_lowercase()
                    .contains(&filter);

                id_match || name_match || source_match
            })
            .take(self.visible_channel_limit)
            .cloned()
            .collect()
    }

    pub(super) fn mix_bus_label(&self, bus_index: usize) -> String {
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

    fn render_mix_matrix(&mut self, ui: &mut egui::Ui) {
        section_header(
            ui,
            self.i18n.text("ui.section.mix_matrix"),
            self.i18n.text("ui.section.mix_matrix_subtitle"),
        );
        ui.add_space(6.0);

        if self.input_channels.is_empty() {
            ui.label(self.i18n.text("ui.inputs.empty"));
            return;
        }

        let visible_channels = self.visible_input_channels();
        if visible_channels.is_empty() {
            ui.label(self.i18n.text("ui.nodes.filter_empty"));
            return;
        }

        egui::ScrollArea::both()
            .id_source("mix_matrix")
            .max_height(360.0)
            .show(ui, |ui| {
                egui::Grid::new("mix_matrix_grid")
                    .striped(true)
                    .min_col_width(112.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(self.i18n.text("ui.label.channels")).strong());
                        for bus_index in 0..self.mix_bus_count {
                            let mut name = self
                                .mix_bus_names
                                .get(bus_index)
                                .cloned()
                                .unwrap_or_else(|| {
                                    Self::default_mix_bus_name(&self.i18n, bus_index)
                                });
                            let response = ui.add_sized(
                                [112.0, 22.0],
                                egui::TextEdit::singleline(&mut name)
                                    .hint_text(self.i18n.text("ui.placeholder.mix_output_name")),
                            );

                            if response.changed() {
                                let trimmed = name.trim();
                                let final_name = if trimmed.is_empty() {
                                    Self::default_mix_bus_name(&self.i18n, bus_index)
                                } else {
                                    trimmed.to_string()
                                };

                                if let Some(slot) = self.mix_bus_names.get_mut(bus_index) {
                                    *slot = final_name;
                                }
                            }
                        }
                        ui.end_row();

                        for channel in &visible_channels {
                            let channel_id = channel.id;
                            let source_node_id = channel.source_node_id;

                            let mut channel_name = channel.name.clone();
                            let source_label = self.source_label(source_node_id);
                            let layout_label = self.source_layout_label(source_node_id);

                            let mut state = self.channel_state.load_or_default(
                                channel_id,
                                Self::default_channel_state(
                                    self.mix_bus_count,
                                    self.source_volume_hint(source_node_id),
                                ),
                            );
                            let mut changed = false;

                            ui.vertical(|ui| {
                                let name_response = ui.add_sized(
                                    [140.0, 22.0],
                                    egui::TextEdit::singleline(&mut channel_name)
                                        .hint_text(self.i18n.text("ui.placeholder.input_name")),
                                );

                                if name_response.changed() {
                                    let final_name = if channel_name.trim().is_empty() {
                                        self.i18n.text_with(
                                            "ui.input.default_name",
                                            &[("index", channel_id.to_string())],
                                        )
                                    } else {
                                        channel_name.trim().to_string()
                                    };

                                    if let Some(entry) = self
                                        .input_channels
                                        .iter_mut()
                                        .find(|entry| entry.id == channel_id)
                                    {
                                        entry.name = final_name;
                                    }
                                }

                                ui.label(
                                    egui::RichText::new(self.i18n.text_with(
                                        "ui.input.layout",
                                        &[("layout", layout_label.clone())],
                                    ))
                                    .small(),
                                );

                                egui::ComboBox::from_id_source(format!(
                                    "source_combo_{}",
                                    channel_id
                                ))
                                .selected_text(source_label)
                                .width(140.0)
                                .show_ui(ui, |ui| {
                                    let mut selected = source_node_id;
                                    if ui
                                        .selectable_label(
                                            selected.is_none(),
                                            self.i18n.text("ui.device.unassigned"),
                                        )
                                        .clicked()
                                    {
                                        selected = None;
                                    }

                                    for node in &self.nodes {
                                        if ui
                                            .selectable_label(
                                                selected == Some(node.id),
                                                format!("#{} {}", node.id, node.name),
                                            )
                                            .clicked()
                                        {
                                            selected = Some(node.id);
                                        }
                                    }

                                    if selected != source_node_id {
                                        if let Some(entry) = self
                                            .input_channels
                                            .iter_mut()
                                            .find(|entry| entry.id == channel_id)
                                        {
                                            entry.source_node_id = selected;
                                        }
                                        self.ensure_input_channel_defaults(channel_id, selected);
                                        changed = true;
                                    }
                                });

                                let mut level_db = Self::gain_to_db(state.level);
                                if ui
                                    .add_sized(
                                        [140.0, 18.0],
                                        egui::Slider::new(
                                            &mut level_db,
                                            FADER_DB_MIN..=FADER_DB_MAX,
                                        )
                                        .show_value(false)
                                        .trailing_fill(true),
                                    )
                                    .changed()
                                {
                                    state.level = Self::db_to_gain(level_db);
                                    changed = true;
                                }

                                ui.label(
                                    egui::RichText::new(self.i18n.text_with(
                                        "ui.input.global_level",
                                        &[(
                                            "value",
                                            Self::format_db(Self::gain_to_db(state.level)),
                                        )],
                                    ))
                                    .small(),
                                );
                                Self::render_zone_meter(ui, state.level, 140.0);
                            });

                            for bus_index in 0..self.mix_bus_count {
                                let Some(send) = state.sends.get_mut(bus_index) else {
                                    continue;
                                };

                                while state.output_mutes.len() <= bus_index {
                                    state.output_mutes.push(false);
                                }

                                let local_mute = state.output_mutes[bus_index];

                                ui.horizontal(|ui| {
                                    let mute_button =
                                        egui::Button::new(self.i18n.text("ui.matrix.local_mute"))
                                            .fill(if local_mute {
                                                egui::Color32::from_rgb(166, 44, 44)
                                            } else {
                                                egui::Color32::from_rgb(46, 56, 74)
                                            });

                                    if ui.add_sized([24.0, 18.0], mute_button).clicked() {
                                        state.output_mutes[bus_index] =
                                            !state.output_mutes[bus_index];
                                        changed = true;
                                    }

                                    let mut send_db = Self::gain_to_db(*send);
                                    if ui
                                        .add_sized(
                                            [62.0, 18.0],
                                            egui::Slider::new(
                                                &mut send_db,
                                                FADER_DB_MIN..=FADER_DB_MAX,
                                            )
                                            .show_value(false)
                                            .trailing_fill(true),
                                        )
                                        .changed()
                                    {
                                        *send = Self::db_to_gain(send_db);
                                        changed = true;
                                    }

                                    ui.vertical(|ui| {
                                        let current = if state.output_mutes[bus_index] {
                                            0.0
                                        } else {
                                            state.level * *send
                                        };

                                        ui.label(
                                            egui::RichText::new(Self::format_db(send_db)).small(),
                                        );
                                        Self::render_zone_meter(ui, current, 36.0);

                                        if layout_label == self.i18n.text("ui.layout.stereo") {
                                            Self::render_zone_meter(
                                                ui,
                                                (current * 0.92).clamp(0.0, 1.0),
                                                36.0,
                                            );
                                        }
                                    });
                                });
                            }

                            if changed {
                                self.channel_state.store(channel_id, state);
                            }

                            ui.end_row();
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

        for (bus_index, level) in mix_levels.buses.iter().enumerate() {
            if bus_index > 0 {
                ui.add_space(6.0);
            }

            ui.label(egui::RichText::new(self.mix_bus_label(bus_index)).strong());
            ui.label(egui::RichText::new(Self::format_db(Self::gain_to_db(*level))).small());
            percent_progress_bar(ui, *level, 220.0, Self::meter_zone_color(*level));
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
                let visible_nodes = self.nodes.clone();
                for node in &visible_nodes {
                    ui.horizontal_wrapped(|ui| {
                        ui.strong(format!("#{}", node.id));
                        ui.label(&node.name);
                        if let Some(ch) = node.channels_hint {
                            ui.label(
                                egui::RichText::new(
                                    self.i18n.text_with(
                                        "ui.node.channels",
                                        &[("count", ch.to_string())],
                                    ),
                                )
                                .small(),
                            );
                        }
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
            self.sync_mix_bus_names();
            self.sync_input_channel_defaults();
            self.render_workspace_controls(ui);
            ui.add_space(10.0);
            self.render_mix_matrix(ui);
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
