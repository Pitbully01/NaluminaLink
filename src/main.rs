use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use eframe::egui;
use pipewire as pw;

mod i18n;

use i18n::I18n;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let i18n = I18n::load();

    match args.get(1).map(String::as_str) {
        Some("ui") => {
            if let Err(error) = run_desktop_ui(&i18n) {
                eprintln!(
                    "{}",
                    i18n.text_with("error.gui", &[("error", error.to_string())])
                );
                std::process::exit(1);
            }
        }
        Some("list-nodes") => {
            if let Err(error) = list_nodes(&i18n) {
                eprintln!(
                    "{}",
                    i18n.text_with("error.list_nodes", &[("error", error.to_string())])
                );
                std::process::exit(1);
            }
        }
        Some("doctor") => {
            println!("{}", i18n.text("doctor.message"));
        }
        Some("help") => {
            print_help(&i18n);
        }
        None => {
            if let Err(error) = run_desktop_ui(&i18n) {
                eprintln!(
                    "{}",
                    i18n.text_with("error.gui", &[("error", error.to_string())])
                );
                std::process::exit(1);
            }
        }
        Some(command) => {
            eprintln!(
                "{}",
                i18n.text_with("error.unknown_command", &[("command", command.to_string())])
            );
            print_help(&i18n);
            std::process::exit(1);
        }
    }
}

fn print_help(i18n: &I18n) {
    println!("{}", i18n.text("cli.subtitle"));
    println!();
    println!("{}", i18n.text("cli.usage"));
    println!("{}", i18n.text("cli.help"));
    println!("{}", i18n.text("cli.doctor"));
    println!("{}", i18n.text("cli.ui"));
    println!("{}", i18n.text("cli.list_nodes"));
}

fn list_nodes(i18n: &I18n) -> Result<(), Box<dyn Error>> {
    let nodes = collect_nodes()?;

    if nodes.is_empty() {
        println!("{}", i18n.text("nodes.empty"));
    } else {
        render_nodes(i18n, &nodes);
    }

    Ok(())
}

fn run_desktop_ui(i18n: &I18n) -> eframe::Result<()> {
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

struct NaluminaApp {
    i18n: I18n,
    nodes: Vec<NodeEntry>,
    status: String,
    refresh_inflight: Option<Receiver<Result<Vec<NodeEntry>, String>>>,
    channel_levels: BTreeMap<u32, f32>,
    channel_mute: BTreeMap<u32, bool>,
    selected_bus: usize,
}

impl NaluminaApp {
    fn new(i18n: I18n) -> Self {
        let mut app = Self {
            status: i18n.text("status.ready"),
            i18n,
            nodes: Vec::new(),
            refresh_inflight: None,
            channel_levels: BTreeMap::new(),
            channel_mute: BTreeMap::new(),
            selected_bus: 0,
        };

        app.start_refresh();
        app
    }

    fn start_refresh(&mut self) {
        if self.refresh_inflight.is_some() {
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.refresh_inflight = Some(receiver);
        self.status = self.i18n.text("status.refreshing_nodes");

        thread::spawn(move || {
            let result = collect_nodes().map_err(|error| error.to_string());
            let _ = sender.send(result);
        });
    }

    fn poll_refresh(&mut self) {
        let Some(receiver) = self.refresh_inflight.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(Ok(nodes)) => {
                self.status = self
                    .i18n
                    .text_with("status.loaded_nodes", &[("count", nodes.len().to_string())]);
                self.nodes = nodes;

                for node in &self.nodes {
                    self.channel_levels.entry(node.id).or_insert(0.72);
                    self.channel_mute.entry(node.id).or_insert(false);
                }
            }
            Ok(Err(error)) => {
                self.status = self
                    .i18n
                    .text_with("status.refresh_failed", &[("error", error)]);
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.refresh_inflight = Some(receiver);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.status = self.i18n.text("status.refresh_disconnected");
            }
        }
    }

    fn apply_theme(ctx: &egui::Context) {
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

    fn draw_channel_strip(&mut self, ui: &mut egui::Ui, node: &NodeEntry) {
        let mut level = *self.channel_levels.entry(node.id).or_insert(0.72);
        let mut muted = *self.channel_mute.entry(node.id).or_insert(false);

        let label = if node.name.len() > 14 {
            format!("{}…", &node.name[..14])
        } else {
            node.name.clone()
        };

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 28, 39))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 62, 88)))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.set_width(128.0);
                ui.vertical_centered(|ui| {
                    ui.strong(label);
                    ui.label(
                        egui::RichText::new(
                            self.i18n
                                .text_with("ui.channel.id", &[("id", node.id.to_string())]),
                        )
                        .size(11.0),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let slider = egui::Slider::new(&mut level, 0.0..=1.0)
                            .vertical()
                            .show_value(false)
                            .trailing_fill(true);
                        ui.add_sized([24.0, 150.0], slider);

                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            let meter_value = if muted { 0.0 } else { level };
                            ui.add(
                                egui::ProgressBar::new(meter_value)
                                    .desired_width(60.0)
                                    .fill(egui::Color32::from_rgb(0, 197, 143))
                                    .text(format!("{:.0}%", meter_value * 100.0)),
                            );

                            if ui
                                .add_sized(
                                    [52.0, 26.0],
                                    egui::Button::new(if muted {
                                        self.i18n.text("ui.channel.unmute")
                                    } else {
                                        self.i18n.text("ui.channel.mute")
                                    }),
                                )
                                .clicked()
                            {
                                muted = !muted;
                            }
                        });
                    });
                });
            });

        self.channel_levels.insert(node.id, level);
        self.channel_mute.insert(node.id, muted);
    }
}

impl eframe::App for NaluminaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_theme(ctx);
        self.poll_refresh();

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
                                self.status = self.i18n.text("doctor.message");
                            }
                        });
                    });
                });
        });

        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.i18n.text("ui.label.status")).strong());
                    ui.label(&self.status);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.i18n.text("ui.section.channel_rack"));
            ui.label(self.i18n.text("ui.section.channel_rack_subtitle"));

            ui.add_space(6.0);

            egui::ScrollArea::horizontal()
                .id_source("channel_rack")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if self.nodes.is_empty() {
                            ui.label(self.i18n.text("nodes.empty"));
                        } else {
                            let visible_nodes: Vec<NodeEntry> =
                                self.nodes.iter().take(10).cloned().collect();
                            for node in &visible_nodes {
                                self.draw_channel_strip(ui, node);
                            }
                        }
                    });
                });

            ui.add_space(14.0);

            ui.columns(2, |columns| {
                columns[0].heading(self.i18n.text("ui.section.routing_buses"));
                columns[0].label(self.i18n.text("ui.section.routing_buses_subtitle"));

                let bus_names = [
                    self.i18n.text("ui.bus.monitor"),
                    self.i18n.text("ui.bus.stream"),
                    self.i18n.text("ui.bus.chat"),
                    self.i18n.text("ui.bus.fx_return"),
                ];
                for (index, name) in bus_names.iter().enumerate() {
                    let selected = self.selected_bus == index;
                    if columns[0]
                        .add(egui::SelectableLabel::new(selected, name))
                        .clicked()
                    {
                        self.selected_bus = index;
                    }
                }

                columns[0].add_space(8.0);
                columns[0].label(
                    egui::RichText::new(self.i18n.text("ui.label.master_bus_level")).strong(),
                );
                let avg_level = if self.channel_levels.is_empty() {
                    0.0
                } else {
                    self.channel_levels.values().sum::<f32>() / self.channel_levels.len() as f32
                };
                columns[0].add(
                    egui::ProgressBar::new(avg_level)
                        .desired_width(220.0)
                        .fill(egui::Color32::from_rgb(0, 168, 255))
                        .text(format!("{:.0}%", avg_level * 100.0)),
                );

                columns[1].heading(self.i18n.text("ui.section.node_browser"));
                columns[1].label(self.i18n.text("ui.section.node_browser_subtitle"));
                egui::ScrollArea::vertical()
                    .id_source("node_browser")
                    .max_height(210.0)
                    .show(&mut columns[1], |ui| {
                        for node in &self.nodes {
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
                    });
            });

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
                        ui.label(match self.selected_bus {
                            0 => self.i18n.text("ui.bus.monitor"),
                            1 => self.i18n.text("ui.bus.stream"),
                            2 => self.i18n.text("ui.bus.chat"),
                            _ => self.i18n.text("ui.bus.fx_return"),
                        });
                    });
                });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }
}

fn collect_nodes() -> Result<Vec<NodeEntry>, Box<dyn Error>> {
    pw::init();

    let nodes = Rc::new(RefCell::new(Vec::new()));

    {
        let main_loop = pw::main_loop::MainLoopRc::new(None)?;
        let context = pw::context::ContextRc::new(&main_loop, None)?;
        let core = context.connect_rc(None)?;
        let registry = core.get_registry_rc()?;

        let main_loop_for_error = main_loop.clone();
        let _core_listener = core
            .add_listener_local()
            .error(move |id, seq, res, message| {
                eprintln!("PipeWire error id={id} seq={seq} res={res}: {message}");
                main_loop_for_error.quit();
            })
            .register();

        let main_loop_for_done = main_loop.clone();
        let sync_seq = core.sync(0)?;
        let nodes_for_global = nodes.clone();

        let _registry_listener = registry
            .add_listener_local()
            .global(move |global| {
                if global.type_ == pw::types::ObjectType::Node {
                    let props = global.props.as_ref();
                    let node_name = props
                        .and_then(|props| props.get("node.name"))
                        .unwrap_or("<unnamed>");
                    let node_description = props
                        .and_then(|props| props.get("node.description"))
                        .unwrap_or("");

                    nodes_for_global.borrow_mut().push(NodeEntry {
                        id: global.id,
                        name: node_name.to_string(),
                        description: node_description.to_string(),
                    });
                }
            })
            .global_remove(|_global_id| {})
            .register();

        let _done_listener = core
            .add_listener_local()
            .done(move |id, seq| {
                if id == pw::core::PW_ID_CORE && seq.seq() == sync_seq.seq() {
                    main_loop_for_done.quit();
                }
            })
            .register();

        main_loop.run();
    }

    unsafe {
        pw::deinit();
    }

    let nodes = match Rc::try_unwrap(nodes) {
        Ok(nodes) => nodes.into_inner(),
        Err(_) => return Err("failed to unwrap collected nodes".into()),
    };

    Ok(nodes)
}

fn render_nodes(i18n: &I18n, nodes: &[NodeEntry]) {
    if nodes.is_empty() {
        println!("{}", i18n.text("nodes.empty"));
        return;
    }

    for node in nodes {
        if node.description.is_empty() {
            println!("{}: {}", node.id, node.name);
        } else {
            println!("{}: {} ({})", node.id, node.name, node.description);
        }
    }
}

#[derive(Clone, Debug)]
struct NodeEntry {
    id: u32,
    name: String,
    description: String,
}
