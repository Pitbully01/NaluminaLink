use eframe::egui;

use super::super::components::{percent_progress_bar, section_header};
use super::super::{InputChannel, NaluminaApp};
use crate::features::ui::state::{MixLevels, MAX_MIX_BUS_COUNT};

const MAX_VISIBLE_CHANNEL_LIMIT: u32 = 24;
const FADER_DB_MIN: f32 = -60.0;
const FADER_DB_MAX: f32 = 0.0;
const MUTE_DB_EPSILON: f32 = 0.001;
const FADER_DB_TICKS: [f32; 11] = [
    -60.0, -54.0, -48.0, -42.0, -36.0, -30.0, -24.0, -18.0, -12.0, -6.0, 0.0,
];

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

    fn db_to_meter_pos(db: f32) -> f32 {
        ((db - FADER_DB_MIN) / (FADER_DB_MAX - FADER_DB_MIN)).clamp(0.0, 1.0)
    }

    fn source_live_levels(&self, source_node_id: Option<u32>) -> (f32, f32) {
        let Some(node_id) = source_node_id else {
            return (0.0, 0.0);
        };

        let Some(node) = self.nodes.iter().find(|node| node.id == node_id) else {
            return (0.0, 0.0);
        };

        if let Some(snapshot) = self.live_meter_store.reading(node_id) {
            return (
                snapshot.current.left.clamp(0.0, 1.0),
                snapshot.current.right.clamp(0.0, 1.0),
            );
        }

        let fallback = node.volume_hint.unwrap_or(0.0).clamp(0.0, 1.0);
        let left = node.peak_left_hint.unwrap_or(fallback).clamp(0.0, 1.0);
        let right = node.peak_right_hint.unwrap_or(left).clamp(0.0, 1.0);
        (left, right)
    }

    fn source_live_level(&self, source_node_id: Option<u32>) -> f32 {
        let (left, right) = self.source_live_levels(source_node_id);
        left.max(right)
    }

    fn source_peak_level(&self, source_node_id: Option<u32>) -> f32 {
        let Some(node_id) = source_node_id else {
            return 0.0;
        };

        if let Some(snapshot) = self.live_meter_store.reading(node_id) {
            return snapshot.peak.left.max(snapshot.peak.right).clamp(0.0, 1.0);
        }

        self.source_live_level(source_node_id)
    }

    fn meter_fill_color_db(db: f32) -> egui::Color32 {
        if db < -18.0 {
            egui::Color32::from_rgb(0, 197, 143)
        } else if db < -6.0 {
            egui::Color32::from_rgb(231, 177, 34)
        } else {
            egui::Color32::from_rgb(219, 68, 55)
        }
    }

    fn meter_zone_color(level: f32) -> egui::Color32 {
        Self::meter_fill_color_db(Self::gain_to_db(level.clamp(0.0, 1.0)))
    }

    fn render_compact_fader(
        ui: &mut egui::Ui,
        level_db: &mut f32,
        live_level: f32,
        peak_level: f32,
        width: f32,
    ) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            let desired_size = egui::vec2(width.max(136.0), 14.0);
            let (rect, response) =
                ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

            if (response.dragged() || response.clicked())
                && response.interact_pointer_pos().is_some()
            {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let t = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                    let next_db = FADER_DB_MIN + t * (FADER_DB_MAX - FADER_DB_MIN);
                    if (next_db - *level_db).abs() > 0.05 {
                        *level_db = next_db;
                        changed = true;
                    }
                }
            }

            let painter = ui.painter_at(rect);
            let rounding = egui::Rounding::same(5.0);

            painter.rect_filled(rect, rounding, egui::Color32::from_rgb(18, 22, 30));

            painter.line_segment(
                [
                    egui::pos2(rect.left(), rect.center().y),
                    egui::pos2(rect.right(), rect.center().y),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0)),
            );

            let live_db = Self::gain_to_db(live_level.clamp(0.0, 1.0));
            let live = Self::db_to_meter_pos(live_db);
            let live_rect = egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.left() + rect.width() * live, rect.bottom()),
            );
            painter.rect_filled(
                live_rect,
                rounding,
                Self::meter_fill_color_db(live_db).linear_multiply(0.78),
            );

            let peak_db = Self::gain_to_db(peak_level.clamp(0.0, 1.0));
            let peak = Self::db_to_meter_pos(peak_db);
            let peak_x = rect.left() + rect.width() * peak;
            painter.line_segment(
                [
                    egui::pos2(peak_x, rect.top() - 2.0),
                    egui::pos2(peak_x, rect.bottom() + 2.0),
                ],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 208, 72)),
            );

            for tick in FADER_DB_TICKS {
                let x = rect.left() + rect.width() * Self::db_to_meter_pos(tick);
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(
                        0.5,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30),
                    ),
                );
            }

            let handle_x = rect.left() + rect.width() * Self::db_to_meter_pos(*level_db);
            let handle_center = egui::pos2(handle_x, rect.center().y);
            let handle_color = if response.dragged() {
                egui::Color32::from_rgb(219, 227, 244)
            } else {
                egui::Color32::from_rgb(189, 199, 220)
            };
            painter.circle_filled(handle_center, 6.0, handle_color);
            painter.circle_stroke(
                handle_center,
                6.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 82, 106)),
            );

            painter.rect_stroke(
                rect,
                rounding,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(68, 80, 100)),
            );
        });

        changed
    }

    fn render_avatar(ui: &mut egui::Ui, label: &str) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        let rounding = egui::Rounding::same(6.0);

        painter.rect_filled(rect, rounding, egui::Color32::from_rgb(25, 31, 42));
        painter.rect_stroke(
            rect,
            rounding,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(76, 87, 108)),
        );

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(230, 236, 245),
        );
    }

    fn avatar_label(name: &str) -> String {
        let letters: String = name
            .chars()
            .filter(|ch| ch.is_ascii_alphabetic())
            .take(2)
            .collect();

        if letters.is_empty() {
            "?".to_string()
        } else {
            letters.to_uppercase()
        }
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

        egui::ScrollArea::vertical()
            .id_source("mix_matrix")
            .max_height(480.0)
            .show(ui, |ui| {
                for channel in &visible_channels {
                    let channel_id = channel.id;
                    let source_node_id = channel.source_node_id;
                    let avatar = Self::avatar_label(&channel.name);

                    let mut channel_name = channel.name.clone();
                    let mut state = self.channel_state.load_or_default(
                        channel_id,
                        Self::default_channel_state(
                            self.mix_bus_count,
                            self.source_volume_hint(source_node_id),
                        ),
                    );
                    let mut changed = false;
                    let live_level = self.source_live_level(source_node_id);
                    let peak_level = self.source_peak_level(source_node_id);

                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(20, 26, 38))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 65, 92)))
                        .rounding(egui::Rounding::same(7.0))
                        .inner_margin(egui::Margin::symmetric(3.0, 2.0))
                        .show(ui, |ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(360.0, 70.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(360.0);

                                    ui.horizontal(|ui| {
                                        Self::render_avatar(ui, &avatar);
                                        ui.add_space(3.0);

                                        ui.vertical(|ui| {
                                            ui.add_sized(
                                                [94.0, 16.0],
                                                egui::TextEdit::singleline(&mut channel_name)
                                                    .hint_text(
                                                        self.i18n.text("ui.placeholder.input_name"),
                                                    ),
                                            );
                                        });

                                        ui.add_space(3.0);

                                        let mute_button = egui::Button::new("M")
                                            .min_size(egui::vec2(16.0, 16.0))
                                            .fill(if state.muted {
                                                egui::Color32::from_rgb(166, 44, 44)
                                            } else {
                                                egui::Color32::from_rgb(46, 56, 74)
                                            });

                                        if ui.add(mute_button).clicked() {
                                            state.muted = !state.muted;
                                            changed = true;
                                        }

                                        ui.add_space(3.0);

                                        ui.vertical(|ui| {
                                            let mut level_db = Self::gain_to_db(state.level);
                                            let slider_width = 152.0;
                                            if Self::render_compact_fader(
                                                ui,
                                                &mut level_db,
                                                live_level,
                                                peak_level,
                                                slider_width,
                                            ) {
                                                state.level = Self::db_to_gain(level_db);
                                                changed = true;
                                            }
                                        });

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("FX").clicked() {
                                                    // placeholder for future filter panel
                                                }
                                            },
                                        );
                                    });
                                },
                            );

                            if changed {
                                self.channel_state.store(channel_id, state);
                            }
                        });

                    ui.add_space(8.0);
                }
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
