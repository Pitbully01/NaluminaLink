use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::features::node_discovery::NodeEntry;
use crate::shared::i18n::I18n;

mod components;
mod refresh;
mod render;
mod state;
mod theme;

use refresh::RefreshResult;
use state::{ChannelStateStore, UiStatus, DEFAULT_MIX_BUS_COUNT, MAX_VISIBLE_CHANNELS};

#[derive(Clone, Debug)]
pub(in crate::features::ui) struct InputChannel {
    pub id: u32,
    pub name: String,
    pub source_node_id: Option<u32>,
}

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
    refresh_inflight: Option<Receiver<RefreshResult>>,
    channel_state: ChannelStateStore,
    input_channels: Vec<InputChannel>,
    next_input_channel_id: u32,
    node_filter: String,
    visible_channel_limit: usize,
    mix_bus_count: usize,
    mix_bus_names: Vec<String>,
    last_auto_refresh: Instant,
}

impl NaluminaApp {
    const AUTO_REFRESH_INTERVAL: Duration = Duration::from_millis(120);

    fn default_input_channels(i18n: &I18n) -> Vec<InputChannel> {
        (0..6)
            .map(|index| InputChannel {
                id: (index + 1) as u32,
                name: i18n.text_with(
                    "ui.input.default_name",
                    &[("index", (index + 1).to_string())],
                ),
                source_node_id: None,
            })
            .collect()
    }

    fn default_mix_bus_name(i18n: &I18n, bus_index: usize) -> String {
        match bus_index {
            0 => i18n.text("ui.bus.monitor"),
            1 => i18n.text("ui.bus.stream"),
            2 => i18n.text("ui.bus.chat"),
            3 => i18n.text("ui.bus.fx_return"),
            _ => i18n.text_with("ui.bus.generic", &[("index", (bus_index + 1).to_string())]),
        }
    }

    fn default_mix_bus_names(i18n: &I18n, count: usize) -> Vec<String> {
        (0..count)
            .map(|index| Self::default_mix_bus_name(i18n, index))
            .collect()
    }

    pub fn new(i18n: I18n) -> Self {
        let mix_bus_count = DEFAULT_MIX_BUS_COUNT;
        let mix_bus_names = Self::default_mix_bus_names(&i18n, mix_bus_count);
        let input_channels = Self::default_input_channels(&i18n);
        let next_input_channel_id = input_channels.len() as u32 + 1;

        let mut app = Self {
            status: UiStatus::new(&i18n),
            i18n,
            nodes: Vec::new(),
            refresh_inflight: None,
            channel_state: ChannelStateStore::new(),
            input_channels,
            next_input_channel_id,
            node_filter: String::new(),
            visible_channel_limit: MAX_VISIBLE_CHANNELS,
            mix_bus_count,
            mix_bus_names,
            last_auto_refresh: Instant::now(),
        };

        app.start_refresh();
        app
    }

    fn maybe_schedule_auto_refresh(&mut self) {
        if self.refresh_inflight.is_some() {
            return;
        }

        if self.last_auto_refresh.elapsed() < Self::AUTO_REFRESH_INTERVAL {
            return;
        }

        self.start_refresh();
        self.last_auto_refresh = Instant::now();
    }
}

impl eframe::App for NaluminaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_theme(ctx);
        self.poll_refresh();
        self.maybe_schedule_auto_refresh();

        self.render_top_bar(ctx);
        self.render_status_bar(ctx);
        self.render_main_panel(ctx);

        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }
}
