use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::Once;

use log::{debug, error, info};
use pipewire as pw;

use crate::i18n::I18n;
use crate::models::NodeEntry;

static PIPEWIRE_INIT: Once = Once::new();

fn ensure_pipewire_init() {
    PIPEWIRE_INIT.call_once(|| {
        debug!("node_discovery: process-wide pipewire init");
        pw::init();
    });
}

pub fn collect_nodes() -> Result<Vec<NodeEntry>, Box<dyn Error>> {
    ensure_pipewire_init();

    let nodes = Rc::new(RefCell::new(Vec::new()));

    {
        let main_loop = pw::main_loop::MainLoopRc::new(None)
            .map_err(|error| format!("pipewire main loop creation failed: {error}"))?;
        let context = pw::context::ContextRc::new(&main_loop, None)
            .map_err(|error| format!("pipewire context creation failed: {error}"))?;
        let core = context
            .connect_rc(None)
            .map_err(|error| format!("pipewire core connection failed: {error}"))?;
        let registry = core
            .get_registry_rc()
            .map_err(|error| format!("pipewire registry acquisition failed: {error}"))?;

        let main_loop_for_error = main_loop.clone();
        let _core_listener = core
            .add_listener_local()
            .error(move |id, seq, res, message| {
                error!(
                    "node_discovery: pipewire core error id={} seq={} res={} message={}",
                    id, seq, res, message
                );
                eprintln!("PipeWire error id={id} seq={seq} res={res}: {message}");
                main_loop_for_error.quit();
            })
            .register();

        let main_loop_for_done = main_loop.clone();
        let sync_seq = core
            .sync(0)
            .map_err(|error| format!("pipewire sync failed: {error}"))?;
        let nodes_for_global = nodes.clone();

        let _registry_listener = registry
            .add_listener_local()
            .global(move |global| {
                if global.type_ != pw::types::ObjectType::Node {
                    return;
                }

                let props = global.props.as_ref();
                let node_name = props
                    .and_then(|properties| properties.get("node.name"))
                    .unwrap_or("<unnamed>");
                let node_description = props
                    .and_then(|properties| properties.get("node.description"))
                    .unwrap_or("");

                nodes_for_global.borrow_mut().push(NodeEntry {
                    id: global.id,
                    name: node_name.to_string(),
                    description: node_description.to_string(),
                });
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

    let nodes = match Rc::try_unwrap(nodes) {
        Ok(nodes) => nodes.into_inner(),
        Err(_) => return Err("failed to unwrap collected nodes".into()),
    };

    info!("node_discovery: collected {} nodes", nodes.len());

    Ok(nodes)
}

pub fn render_nodes(i18n: &I18n, nodes: &[NodeEntry]) {
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
