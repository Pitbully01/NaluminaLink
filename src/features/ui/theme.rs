use eframe::egui;

use super::NaluminaApp;

impl NaluminaApp {
    pub(super) fn apply_theme(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(18, 22, 32);
        style.visuals.window_fill = egui::Color32::from_rgb(13, 18, 28);
        style.visuals.panel_fill = egui::Color32::from_rgb(10, 14, 22);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(29, 37, 52);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(41, 59, 86);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(54, 88, 132);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 168, 255);
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 232, 245));
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        ctx.set_style(style);
    }
}
