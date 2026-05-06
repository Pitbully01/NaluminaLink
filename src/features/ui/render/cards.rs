use eframe::egui;

use super::super::state::{ChannelStripState, MixLevels};
use super::super::{InputChannel, NaluminaApp};
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

fn default_card_fill() -> egui::Color32 {
    egui::Color32::from_rgb(28, 35, 44)
}

fn default_card_stroke() -> egui::Color32 {
    egui::Color32::from_rgb(66, 82, 103)
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
        let mut out = s.chars().take(max_len.saturating_sub(1)).collect::<String>();
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

fn render_channel_labels(
    app: &mut NaluminaApp,
    ui: &mut egui::Ui,
    channel_id: u32,
    channel_name: &str,
    source_label: &str,
    node_choices: &[NodeChoice],
) {
    ui.label(egui::RichText::new(channel_name).size(12.0).strong());
    render_source_picker(app, ui, channel_id, source_label, node_choices);
}

fn render_channel_controls(
    ui: &mut egui::Ui,
    state: &mut ChannelStripState,
    live_level: f32,
    peak_level: f32,
) -> bool {
    let mut changed = false;

    let mute_button = egui::Button::new(if state.muted { "🔇" } else { "🔊" })
        .min_size(egui::vec2(28.0, 20.0))
        .fill(if state.muted {
            egui::Color32::from_rgb(166, 44, 44)
        } else {
            egui::Color32::from_rgb(58, 67, 82)
        });

    if ui.add(mute_button).clicked() {
        state.muted = !state.muted;
        changed = true;
    }

    ui.add_space(6.0);

    let mut level_db = gain_to_db(state.level);
    if render_compact_fader(ui, &mut level_db, live_level, peak_level, 132.0) {
        state.level = db_to_gain(level_db);
        changed = true;
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let fx_button = egui::Button::new("FX")
            .min_size(egui::vec2(26.0, 16.0))
            .fill(egui::Color32::from_rgb(58, 67, 82));
        if ui.add(fx_button).clicked() {
            // placeholder for future filter panel
        }
    });

    changed
}

fn render_channel_card(
    app: &mut NaluminaApp,
    ui: &mut egui::Ui,
    channel: &InputChannel,
    node_choices: &[NodeChoice],
) {
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

    let mut changed = false;

    ui.allocate_ui_with_layout(
        channel_card_size(),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_min_size(channel_card_size());
            ui.set_max_size(channel_card_size());

            channel_card_frame(default_card_fill(), default_card_stroke()).show(ui, |ui| {
                ui.set_min_size(channel_card_inner_size());
                ui.set_max_size(channel_card_inner_size());

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let avatar = avatar_label(&channel.name);
                    render_avatar(ui, &avatar);
                    ui.add_space(6.0);

                    ui.vertical(|ui| {
                        render_channel_labels(
                            app,
                            ui,
                            channel.id,
                            &channel.name,
                            &source_label,
                            node_choices,
                        );
                        ui.add_space(4.0);
                        let (live_left, live_right) = app.source_live_levels(source_node_id);
                        render_lr_meter(ui, live_left, live_right);
                    });

                    ui.add_space(6.0);

                    if render_channel_controls(ui, &mut state, live_level, peak_level) {
                        changed = true;
                    }
                });
            });
        },
    );

    if changed {
        app.channel_state.store(channel.id, state);
    }
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

    egui::ScrollArea::vertical()
        .id_source("mix_matrix")
        .max_height(480.0)
        .show(ui, |ui| {
            for channel in &visible_channels {
                render_channel_card(app, ui, channel, node_choices);
                ui.add_space(6.0);
            }
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
