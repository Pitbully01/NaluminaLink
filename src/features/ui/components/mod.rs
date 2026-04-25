use eframe::egui;

pub(in crate::features::ui) fn section_header(ui: &mut egui::Ui, title: String, subtitle: String) {
    ui.heading(title);
    ui.label(subtitle);
}

pub(in crate::features::ui) fn percent_progress_bar(
    ui: &mut egui::Ui,
    value: f32,
    width: f32,
    fill: egui::Color32,
) {
    ui.add(
        egui::ProgressBar::new(value)
            .desired_width(width)
            .fill(fill)
            .text(format!("{:.0}%", value * 100.0)),
    );
}
