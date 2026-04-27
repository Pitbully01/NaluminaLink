use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::Once;

use log::{debug, error, info};
use pipewire as pw;

use crate::shared::i18n::I18n;

use super::domain::NodeEntry;

static PIPEWIRE_INIT: Once = Once::new();

fn parse_volume_hint(props: Option<&pw::spa::utils::dict::DictRef>) -> Option<f32> {
    let keys = ["volume", "node.volume", "audio.volume", "channelmix.volume"];

    keys.iter().find_map(|key| {
        props
            .and_then(|properties| properties.get(key))
            .and_then(|raw| raw.trim().parse::<f32>().ok())
    })
}

fn parse_channels_hint(props: Option<&pw::spa::utils::dict::DictRef>) -> Option<u8> {
    let keys = ["audio.channels", "channel.count", "node.channels"];

    keys.iter().find_map(|key| {
        props
            .and_then(|properties| properties.get(key))
            .and_then(|raw| raw.trim().parse::<u8>().ok())
    })
}

fn parse_peak_hint(props: Option<&pw::spa::utils::dict::DictRef>, keys: &[&str]) -> Option<f32> {
    keys.iter().find_map(|key| {
        props
            .and_then(|properties| properties.get(key))
            .and_then(|raw| parse_float_tokens(raw).first().copied())
            .map(|value| value.clamp(0.0, 1.0))
    })
}

fn parse_peak_channel_hint(
    props: Option<&pw::spa::utils::dict::DictRef>,
    keys: &[&str],
    channel_index: usize,
) -> Option<f32> {
    keys.iter().find_map(|key| {
        props
            .and_then(|properties| properties.get(key))
            .and_then(|raw| parse_float_tokens(raw).get(channel_index).copied())
            .map(|value| value.clamp(0.0, 1.0))
    })
}

fn parse_float_tokens(raw: &str) -> Vec<f32> {
    let mut values = Vec::new();
    let mut token = String::new();

    for ch in raw.chars() {
        if ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E') {
            token.push(ch);
            continue;
        }

        if !token.is_empty() {
            if let Ok(value) = token.parse::<f32>() {
                values.push(value);
            }
            token.clear();
        }
    }

    if !token.is_empty() {
        if let Ok(value) = token.parse::<f32>() {
            values.push(value);
        }
    }

    values
}

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

                let props = global.props.as_deref();
                let node_name = props
                    .and_then(|properties| properties.get("node.name"))
                    .unwrap_or("<unnamed>");
                let node_description = props
                    .and_then(|properties| properties.get("node.description"))
                    .unwrap_or("");
                let volume_hint = parse_volume_hint(props);
                let channels_hint = parse_channels_hint(props);
                let peak_left_hint = parse_peak_hint(
                    props,
                    &[
                        "audio.peak.left",
                        "peak.left",
                        "peak.l",
                        "meter.left",
                        "monitor.peak.left",
                    ],
                );
                let peak_right_hint = parse_peak_hint(
                    props,
                    &[
                        "audio.peak.right",
                        "peak.right",
                        "peak.r",
                        "meter.right",
                        "monitor.peak.right",
                    ],
                )
                .or_else(|| {
                    parse_peak_channel_hint(
                        props,
                        &["audio.peak", "peak", "monitor.peak", "monitor.channel-peaks"],
                        1,
                    )
                });

                let peak_left_hint = peak_left_hint.or_else(|| {
                    parse_peak_channel_hint(
                        props,
                        &["audio.peak", "peak", "monitor.peak", "monitor.channel-peaks"],
                        0,
                    )
                });

                nodes_for_global.borrow_mut().push(NodeEntry {
                    id: global.id,
                    name: node_name.to_string(),
                    description: node_description.to_string(),
                    volume_hint,
                    channels_hint,
                    peak_left_hint,
                    peak_right_hint,
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
