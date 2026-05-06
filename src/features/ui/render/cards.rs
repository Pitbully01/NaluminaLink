use eframe::egui;

use super::super::state::MixLevels;
use super::super::NaluminaApp;
use super::widgets::{
    db_to_gain, format_db, gain_to_db, meter_zone_color, render_avatar, render_compact_fader,
    render_lr_meter,
};

const CARD_WIDTH: f32 = 360.0;
const CARD_HEIGHT: f32 = 70.0;
const CARD_INNER_MARGIN_X: f32 = 2.0;
const CARD_INNER_MARGIN_Y: f32 = 1.0;
const CARD_OUTER_INNER_WIDTH: f32 = 356.0;
const CARD_OUTER_INNER_HEIGHT: f32 = 66.0;
const CHANNEL_NAME_WIDTH: usize = 28;
const MATRIX_ROW_LABEL_WIDTH: f32 = 260.0;
const MATRIX_BUS_COL_WIDTH: f32 = 172.0;
const MATRIX_BUS_HEADER_HEIGHT: f32 = 54.0;
const MATRIX_ROW_HEIGHT: f32 = 66.0;

#[derive(Clone)]
pub(crate) struct NodeChoice {
    pub id: u32,
    pub name: String,
}

fn channel_card_size() -> egui::Vec2 {
    egui::vec2(CARD_WIDTH, CARD_HEIGHT)
}

fn channel_card_inner_size() -> egui::Vec2 {
    egui::vec2(CARD_OUTER_INNER_WIDTH, CARD_OUTER_INNER_HEIGHT)
}

fn channel_card_frame(fill: egui::Color32, stroke: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(
            CARD_INNER_MARGIN_X,
            CARD_INNER_MARGIN_Y,
        ))
}

fn output_card_fill() -> egui::Color32 {
    egui::Color32::from_rgb(20, 26, 38)
}

fn output_card_stroke() -> egui::Color32 {
    egui::Color32::from_rgb(50, 65, 92)
}

fn truncate_label(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut out = s
            .chars()
            .take(max_len.saturating_sub(1))
            .collect::<String>();
        out.push('…');
        out
    }
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

fn source_choice_label(source_label: &str) -> String {
    truncate_label(source_label, CHANNEL_NAME_WIDTH)
}

fn render_source_picker(
    app: &mut NaluminaApp,
    ui: &mut egui::Ui,
    channel_id: u32,
    source_label: &str,
    node_choices: &[NodeChoice],
) {
    let label = source_choice_label(source_label);
    ui.menu_button(
        egui::RichText::new(label)
            .size(10.0)
            .color(egui::Color32::from_rgb(155, 170, 188)),
        |ui| {
            for choice in node_choices {
                if ui.button(&choice.name).clicked() {
                    set_channel_source(app, channel_id, Some(choice.id));
                    ui.close_menu();
                }
            }

            if ui.button(app.i18n.text("ui.device.unassigned")).clicked() {
                set_channel_source(app, channel_id, None);
                ui.close_menu();
            }
        },
    );
}

fn set_channel_source(app: &mut NaluminaApp, channel_id: u32, source_node_id: Option<u32>) {
    if let Some(channel) = app
        .input_channels
        .iter_mut()
        .find(|channel| channel.id == channel_id)
    {
        channel.source_node_id = source_node_id;
        app.sync_live_meter_sources();
    }
}

fn render_matrix_bus_header(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(29, 36, 46))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 72, 94)))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(MATRIX_BUS_COL_WIDTH, MATRIX_BUS_HEADER_HEIGHT));
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(text).size(12.0).strong());
                ui.add_space(4.0);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(104.0, 2.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 1.0, egui::Color32::from_rgb(77, 208, 122));
            });
        });
}

fn render_matrix_send_cell(
    ui: &mut egui::Ui,
    bus_name: &str,
    value: &mut f32,
    live_level: f32,
    peak_level: f32,
) -> bool {
    let mut changed = false;

    egui::Frame::none()
        .fill(egui::Color32::from_rgb(27, 34, 43))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 72, 94)))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_width(MATRIX_BUS_COL_WIDTH);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("🔊")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(216, 224, 235)),
                );
                ui.label(
                    egui::RichText::new(bus_name)
                        .size(9.5)
                        .color(egui::Color32::from_rgb(155, 170, 188)),
                );
                ui.add_space(4.0);

                let mut send_db = gain_to_db(*value);
                if render_compact_fader(ui, &mut send_db, live_level, peak_level, 136.0) {
                    *value = db_to_gain(send_db);
                    changed = true;
                }
            });
        });

    changed
}

fn render_output_card(app: &NaluminaApp, ui: &mut egui::Ui, bus_index: usize, level: f32) {
    let bus_name = app.mix_bus_label(bus_index);
    let avatar = avatar_label(&bus_name);

    ui.allocate_ui_with_layout(
        channel_card_size(),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_min_size(channel_card_size());
            ui.set_max_size(channel_card_size());

            channel_card_frame(output_card_fill(), output_card_stroke()).show(ui, |ui| {
                ui.set_min_size(channel_card_inner_size());
                ui.set_max_size(channel_card_inner_size());

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    render_avatar(ui, &avatar);
                    ui.add_space(4.0);

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(bus_name).size(13.0).strong());
                        ui.label(egui::RichText::new(format_db(gain_to_db(level))).size(11.0));
                    });

                    ui.add_space(6.0);
                    super::super::components::percent_progress_bar(
                        ui,
                        level,
                        170.0,
                        meter_zone_color(level),
                    );
                });
            });
        },
    );
}

pub(in crate::features::ui) fn render_mix_matrix(
    app: &mut NaluminaApp,
    ui: &mut egui::Ui,
    node_choices: &[NodeChoice],
) {
    super::super::components::section_header(
        ui,
        app.i18n.text("ui.section.mix_matrix"),
        app.i18n.text("ui.section.mix_matrix_subtitle"),
    );
    ui.add_space(6.0);

    if app.input_channels.is_empty() {
        ui.label(app.i18n.text("ui.inputs.empty"));
        return;
    }

    let visible_channels = app.visible_input_channels();
    if visible_channels.is_empty() {
        ui.label(app.i18n.text("ui.nodes.filter_empty"));
        return;
    }

    egui::Frame::none()
        .fill(egui::Color32::from_rgb(16, 22, 32))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(44, 58, 79)))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(8.0, 8.0))
        .show(ui, |ui| {
            egui::ScrollArea::both()
                .id_source("mix_matrix")
                .max_height(540.0)
                .show(ui, |ui| {
                    // HEADER ROW: Bus headers + add button
                    ui.horizontal(|ui| {
                        ui.add_space(MATRIX_ROW_LABEL_WIDTH);
                        
                        for bus_index in 0..app.mix_bus_count {
                            render_matrix_bus_header(ui, &app.mix_bus_label(bus_index));
                            ui.add_space(8.0);
                        }

                        // Add new mix bus button
                        if app.mix_bus_count < crate::features::ui::state::MAX_MIX_BUS_COUNT {
                            if ui
                                .button(
                                    egui::RichText::new("+")
                                        .size(16.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(77, 208, 122)),
                                )
                                .clicked()
                            {
                                app.mix_bus_count = (app.mix_bus_count + 1)
                                    .min(crate::features::ui::state::MAX_MIX_BUS_COUNT);
                                app.sync_mix_bus_names();
                            }
                        }
                    });

                    ui.add_space(12.0);

                    // DATA ROWS: Each input channel + bridge cells
                    for channel in &visible_channels {
                        ui.horizontal(|ui| {
                            let source_node_id = channel.source_node_id;
                            let source_label = app.source_label(source_node_id);
                            let live_level = app.source_live_level(source_node_id);
                            let peak_level = app.source_peak_level(source_node_id);

                            let mut state = app.channel_state.load_or_default(
                                channel.id,
                                NaluminaApp::default_channel_state(
                                    app.mix_bus_count,
                                    app.source_volume_hint(source_node_id),
                                ),
                            );

                            // LEFT COLUMN: Channel info
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(24, 31, 40))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 76, 96)))
                                .rounding(egui::Rounding::same(6.0))
                                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                                .show(ui, |ui| {
                                    ui.set_min_size(egui::vec2(
                                        MATRIX_ROW_LABEL_WIDTH,
                                        MATRIX_ROW_HEIGHT,
                                    ));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            let avatar = avatar_label(&channel.name);
                                            render_avatar(ui, &avatar);
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(&channel.name)
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        });

                                        ui.add_space(2.0);
                                        render_source_picker(
                                            app,
                                            ui,
                                            channel.id,
                                            &source_label,
                                            node_choices,
                                        );
                                        ui.add_space(2.0);
                                        let (live_left, live_right) =
                                            app.source_live_levels(source_node_id);
                                        render_lr_meter(ui, live_left, live_right);

                                        ui.add_space(4.0);

                                        ui.horizontal(|ui| {
                                            // Mute Button
                                            let mute_color = if state.muted {
                                                egui::Color32::from_rgb(255, 107, 107)
                                            } else {
                                                egui::Color32::from_rgb(110, 130, 160)
                                            };
                                            if ui
                                                .button(
                                                    egui::RichText::new("🔇")
                                                        .size(11.0)
                                                        .color(mute_color),
                                                )
                                                .clicked()
                                            {
                                                state.muted = !state.muted;
                                            }

                                            ui.add_space(4.0);

                                            // FX Button
                                            if ui
                                                .button(
                                                    egui::RichText::new("FX")
                                                        .size(9.0)
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(155, 170, 188)),
                                                )
                                                .clicked()
                                            {
                                                // TODO: Open FX panel
                                            }
                                        });
                                    });
                                });

                            ui.add_space(8.0);

                            // MATRIX CELLS: Bridge icons for each bus
                            for bus_index in 0..app.mix_bus_count {
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(27, 34, 43))
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 72, 94)))
                                    .rounding(egui::Rounding::same(6.0))
                                    .inner_margin(egui::Margin::symmetric(8.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.set_min_size(egui::vec2(MATRIX_BUS_COL_WIDTH, MATRIX_ROW_HEIGHT));
                                        ui.vertical_centered(|ui| {
                                            // Bridge/Link Icon
                                            if ui
                                                .button(
                                                    egui::RichText::new("+")
                                                        .size(20.0)
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(77, 208, 122)),
                                                )
                                                .clicked()
                                            {
                                                // TODO: Create bridge link
                                            }
                                            ui.label(
                                                egui::RichText::new("Bridge")
                                                    .size(8.0)
                                                    .color(egui::Color32::from_rgb(155, 170, 188)),
                                            );
                                        });
                                    });

                                ui.add_space(8.0);
                            }

                            app.channel_state.store(channel.id, state);
                        });

                        ui.add_space(12.0);
                    }

                    // FOOTER ROW: Add input channel button
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("+ Eingang")
                                    .size(12.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(77, 208, 122)),
                            )
                            .clicked()
                        {
                            app.add_input_channel();
                        }
                    });
                });
        });
}

pub(in crate::features::ui) fn render_mix_outputs(
    app: &NaluminaApp,
    ui: &mut egui::Ui,
    mix_levels: &MixLevels,
) {
    super::super::components::section_header(
        ui,
        app.i18n.text("ui.section.mix_outputs"),
        app.i18n.text("ui.section.mix_outputs_subtitle"),
    );
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .id_source("mix_outputs")
        .max_height(280.0)
        .show(ui, |ui| {
            for (bus_index, level) in mix_levels.buses.iter().enumerate() {
                render_output_card(app, ui, bus_index, *level);
                ui.add_space(6.0);
            }
        });
}
