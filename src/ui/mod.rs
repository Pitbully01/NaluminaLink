use std::sync::mpsc::Receiver;

use eframe::egui;

use crate::i18n::I18n;
use crate::models::NodeEntry;

mod components;
mod refresh;
mod render;
mod state;
mod theme;

use state::{ChannelStateStore, UiStatus};

pub fn run_desktop_ui(i18n: &I18n) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([920.0, 640.0]),
        ..Default::default()
    };

    let i18n_for_app = i18n.clone();

    eframe::run_native(
        &i18n.text("app.window_title"),
        options,
        Box::new(move |_cc| Ok(Box::new(NaluminaApp::new(i18n_for_app.clone())))),
    )
}

pub struct NaluminaApp {
    i18n: I18n,
    nodes: Vec<NodeEntry>,
    status: UiStatus,
    refresh_inflight: Option<Receiver<Result<Vec<NodeEntry>, String>>>,
    channel_state: ChannelStateStore,
}

impl NaluminaApp {
    pub fn new(i18n: I18n) -> Self {
        let mut app = Self {
            status: UiStatus::new(&i18n),
            i18n,
            nodes: Vec::new(),
            refresh_inflight: None,
            channel_state: ChannelStateStore::new(),
        };

        app.start_refresh();
        app
    }
}

impl eframe::App for NaluminaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_theme(ctx);
        self.poll_refresh();

        self.render_top_bar(ctx);
        self.render_status_bar(ctx);
        self.render_main_panel(ctx);

        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }
}
