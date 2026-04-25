use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use eframe::egui;
use pipewire as pw;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("ui") => {
            if let Err(error) = run_desktop_ui() {
                eprintln!("GUI error: {error}");
                std::process::exit(1);
            }
        }
        Some("list-nodes") => {
            if let Err(error) = list_nodes() {
                eprintln!("Failed to list PipeWire nodes: {error}");
                std::process::exit(1);
            }
        }
        Some("doctor") => {
            println!("naluminaLink is installed and ready to talk to PipeWire later.");
        }
        Some("help") => {
            print_help();
        }
        None => {
            if let Err(error) = run_desktop_ui() {
                eprintln!("GUI error: {error}");
                std::process::exit(1);
            }
        }
        Some(command) => {
            eprintln!("Unknown command: {command}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("naluminaLink - PipeWire audio router");
    println!();
    println!("Usage:");
    println!("  naluminaLink help");
    println!("  naluminaLink doctor");
    println!("  naluminaLink ui   # open the desktop UI");
    println!("  naluminaLink list-nodes");
}

fn list_nodes() -> Result<(), Box<dyn Error>> {
    let nodes = collect_nodes()?;

    if nodes.is_empty() {
        println!("No PipeWire nodes found.");
    } else {
        render_nodes(&nodes);
    }

    Ok(())
}

fn run_desktop_ui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([920.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "naluminaLink",
        options,
        Box::new(|_cc| Ok(Box::new(NaluminaApp::new()))),
    )
}

struct NaluminaApp {
    nodes: Vec<NodeEntry>,
    status: String,
    refresh_inflight: Option<Receiver<Result<Vec<NodeEntry>, String>>>,
}

impl NaluminaApp {
    fn new() -> Self {
        let mut app = Self {
            nodes: Vec::new(),
            status: String::from("Ready."),
            refresh_inflight: None,
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
        self.status = String::from("Refreshing PipeWire nodes...");

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
                self.status = format!("Loaded {} PipeWire nodes.", nodes.len());
                self.nodes = nodes;
            }
            Ok(Err(error)) => {
                self.status = format!("Refresh failed: {error}");
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.refresh_inflight = Some(receiver);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.status = String::from("Refresh worker disconnected.");
            }
        }
    }
}

impl eframe::App for NaluminaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_refresh();

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("naluminaLink");
                ui.label("PipeWire audio router test UI");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(&self.status);
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Refresh nodes").clicked() {
                    self.start_refresh();
                }

                if ui.button("Doctor").clicked() {
                    self.status = String::from(
                        "naluminaLink is installed and ready to talk to PipeWire later.",
                    );
                }
            });

            ui.add_space(16.0);
            ui.heading("PipeWire Nodes");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.nodes.is_empty() {
                    ui.label("No PipeWire nodes found.");
                } else {
                    for node in &self.nodes {
                        ui.group(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.strong(format!("{}", node.id));
                                ui.label(&node.name);
                            });

                            if !node.description.is_empty() {
                                ui.label(&node.description);
                            }
                        });
                    }
                }
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

fn render_nodes(nodes: &[NodeEntry]) {
    if nodes.is_empty() {
        println!("No PipeWire nodes found.");
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
