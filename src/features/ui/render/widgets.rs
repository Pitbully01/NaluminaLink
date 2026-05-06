use eframe::egui;

const FADER_DB_MIN: f32 = -60.0;
const FADER_DB_MAX: f32 = 0.0;
const MUTE_DB_EPSILON: f32 = 0.001;

pub(in crate::features::ui) fn gain_to_db(gain: f32) -> f32 {
    if gain <= MUTE_DB_EPSILON {
        FADER_DB_MIN
    } else {
        (20.0 * gain.log10()).clamp(FADER_DB_MIN, FADER_DB_MAX)
    }
}

pub(in crate::features::ui) fn db_to_gain(db: f32) -> f32 {
    if db <= FADER_DB_MIN + 0.05 {
        0.0
    } else {
        10f32.powf(db / 20.0)
    }
}

pub(in crate::features::ui) fn format_db(db: f32) -> String {
    if db <= FADER_DB_MIN + 0.05 {
        "-inf dB".to_string()
    } else {
        format!("{db:.1} dB")
    }
}

pub(in crate::features::ui) fn db_to_meter_pos(db: f32) -> f32 {
    ((db - FADER_DB_MIN) / (FADER_DB_MAX - FADER_DB_MIN)).clamp(0.0, 1.0)
}

pub(in crate::features::ui) fn meter_fill_color_db(db: f32) -> egui::Color32 {
    if db < -18.0 {
        egui::Color32::from_rgb(46, 197, 105)
    } else if db < -6.0 {
        egui::Color32::from_rgb(231, 177, 34)
    } else {
        egui::Color32::from_rgb(219, 68, 55)
    }
}

pub(in crate::features::ui) fn meter_zone_color(level: f32) -> egui::Color32 {
    meter_fill_color_db(gain_to_db(level.clamp(0.0, 1.0)))
}

pub(in crate::features::ui) fn render_compact_fader(
    ui: &mut egui::Ui,
    level_db: &mut f32,
    live_level: f32,
    peak_level: f32,
    width: f32,
) -> bool {
    let mut changed = false;
    ui.vertical(|ui| {
        let desired_size = egui::vec2(width.max(132.0), 12.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        if (response.dragged() || response.clicked()) && response.interact_pointer_pos().is_some() {
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
        let rounding = egui::Rounding::same(4.0);

        painter.rect_filled(rect, rounding, egui::Color32::from_rgb(34, 40, 46));

        let live_db = gain_to_db(live_level.clamp(0.0, 1.0));
        let live = db_to_meter_pos(live_db);
        let bar_h = rect.height().min(6.0);
        let live_bar = egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.center().y - bar_h / 2.0),
            egui::pos2(rect.left() + rect.width() * live, rect.center().y + bar_h / 2.0),
        );
        painter.rect_filled(
            live_bar,
            egui::Rounding::same(3.0),
            egui::Color32::from_rgba_unmultiplied(46, 197, 105, 220),
        );

        let peak_db = gain_to_db(peak_level.clamp(0.0, 1.0));
        let peak = db_to_meter_pos(peak_db);
        let peak_x = rect.left() + rect.width() * peak;
        painter.line_segment(
            [
                egui::pos2(peak_x, rect.top()),
                egui::pos2(peak_x, rect.bottom()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(210, 240, 222)),
        );

        let handle_x = rect.left() + rect.width() * db_to_meter_pos(*level_db);
        let handle_center = egui::pos2(handle_x, rect.center().y);
        let handle_color = if response.dragged() {
            egui::Color32::from_rgb(236, 242, 248)
        } else {
            egui::Color32::from_rgb(216, 224, 235)
        };
        painter.circle_filled(handle_center, 3.0, handle_color);
        painter.circle_stroke(
            handle_center,
            3.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(85, 95, 112)),
        );

        painter.rect_stroke(
            rect,
            rounding,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(62, 74, 92)),
        );
    });

    changed
}

pub(in crate::features::ui) fn render_lr_meter(ui: &mut egui::Ui, left: f32, right: f32) {
    let size = egui::vec2(46.0, 12.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let rounding = egui::Rounding::same(4.0);

    painter.rect_filled(rect, rounding, egui::Color32::from_rgb(33, 40, 48));

    let gap = 6.0;
    let half = (rect.width() - gap) / 2.0;

    let left_w = half * left.clamp(0.0, 1.0);
    let right_w = half * right.clamp(0.0, 1.0);

    let left_rect = egui::Rect::from_min_max(
        rect.left_top(),
        egui::pos2(rect.left() + left_w, rect.bottom()),
    );
    let right_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + half + gap, rect.top()),
        egui::pos2(rect.left() + half + gap + right_w, rect.bottom()),
    );

    painter.rect_filled(left_rect, rounding, egui::Color32::from_rgb(46, 197, 105));
    painter.rect_filled(right_rect, rounding, egui::Color32::from_rgb(46, 197, 105));
}

pub(in crate::features::ui) fn render_avatar(ui: &mut egui::Ui, label: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let rounding = egui::Rounding::same(4.0);

    painter.rect_filled(rect, rounding, egui::Color32::from_rgb(231, 236, 242));
    painter.rect_stroke(
        rect,
        rounding,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(130, 141, 161)),
    );

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.0),
        egui::Color32::from_rgb(42, 50, 66),
    );
}
