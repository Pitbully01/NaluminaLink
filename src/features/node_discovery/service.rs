use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::process::Command;
use std::rc::Rc;
use std::sync::Once;

use log::{debug, error};
use pipewire as pw;

use crate::shared::i18n::I18n;

use super::domain::NodeEntry;

static PIPEWIRE_INIT: Once = Once::new();

fn parse_volume_hint(props: Option<&pw::spa::utils::dict::DictRef>) -> Option<f32> {
    let keys = ["volume", "node.volume", "audio.volume", "channelmix.volume"];

    keys.iter().find_map(|key| {
        props
            .and_then(|properties| properties.get(key))
            .and_then(|raw| parse_float_tokens(raw).first().copied())
            .map(normalize_gain_hint)
    })
}

fn parse_channels_hint(props: Option<&pw::spa::utils::dict::DictRef>) -> Option<u8> {
    let numeric_keys = ["audio.channels", "channel.count", "node.channels"];

    let numeric_hint = numeric_keys.iter().find_map(|key| {
        props
            .and_then(|properties| properties.get(key))
            .and_then(|raw| parse_float_tokens(raw).first().copied())
            .map(|value| value.round().clamp(1.0, 64.0) as u8)
    });

    if numeric_hint.is_some() {
        return numeric_hint;
    }

    let map_keys = ["audio.position", "audio.positions", "audio.channel-map"];

    map_keys.iter().find_map(|key| {
        props
            .and_then(|properties| properties.get(key))
            .and_then(|raw| {
                let labels = parse_channel_labels(raw);
                if labels.is_empty() {
                    None
                } else if labels.iter().any(|label| label == "MONO") {
                    Some(1)
                } else if labels.iter().any(|label| label == "STEREO") {
                    Some(2)
                } else if labels.len() == 1 {
                    None
                } else {
                    Some(labels.len().clamp(1, 64) as u8)
                }
            })
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

fn parse_channel_labels(raw: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let mut token = String::new();

    for ch in raw.chars() {
        if ch.is_ascii_alphabetic() || ch == '_' {
            token.push(ch.to_ascii_uppercase());
            continue;
        }

        if !token.is_empty() {
            labels.push(token.clone());
            token.clear();
        }
    }

    if !token.is_empty() {
        labels.push(token);
    }

    labels
}

fn normalize_gain_hint(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }

    if value > 1.0 {
        if value <= 100.0 {
            (value / 100.0).clamp(0.0, 1.0)
        } else {
            1.0
        }
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn probe_source_levels(node_id: u32, channels_hint: Option<u8>) -> Option<(f32, f32)> {
    let channels = channels_hint.unwrap_or(2).clamp(1, 2);
    let sample_count = u32::from(channels) * 2048;

    let output = Command::new("timeout")
        .args([
            "--signal=KILL",
            "0.20s",
            "pw-cat",
            "--record",
            "--raw",
            "--target",
            &node_id.to_string(),
            "--format",
            "s16",
            "--channels",
            &channels.to_string(),
            "--sample-count",
            &sample_count.to_string(),
            "-",
        ])
        .output()
        .ok()?;

    if output.stdout.len() < 4 {
        return None;
    }

    let mut left_peak = 0.0_f32;
    let mut right_peak = 0.0_f32;
    let stride = usize::from(channels) * 2;

    for frame in output.stdout.chunks_exact(stride) {
        let left = i16::from_le_bytes([frame[0], frame[1]]) as f32 / i16::MAX as f32;
        left_peak = left_peak.max(left.abs());

        if channels > 1 {
            let right = i16::from_le_bytes([frame[2], frame[3]]) as f32 / i16::MAX as f32;
            right_peak = right_peak.max(right.abs());
        }
    }

    if channels == 1 {
        right_peak = left_peak;
    }

    Some((left_peak.clamp(0.0, 1.0), right_peak.clamp(0.0, 1.0)))
}

fn ensure_pipewire_init() {
    PIPEWIRE_INIT.call_once(|| {
        debug!("node_discovery: process-wide pipewire init");
        pw::init();
    });
}

pub fn collect_nodes() -> Result<Vec<NodeEntry>, Box<dyn Error>> {
    collect_nodes_for_sources(&HashSet::new())
}

pub fn collect_nodes_for_sources(
    probed_source_ids: &HashSet<u32>,
) -> Result<Vec<NodeEntry>, Box<dyn Error>> {
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
                        &[
                            "audio.peak",
                            "peak",
                            "monitor.peak",
                            "monitor.channel-peaks",
                        ],
                        1,
                    )
                })
                .or_else(|| {
                    parse_peak_channel_hint(
                        props,
                        &[
                            "monitor.channel-volumes",
                            "channelmix.volumes",
                            "audio.channel.volumes",
                        ],
                        1,
                    )
                });

                let peak_left_hint = peak_left_hint
                    .or_else(|| {
                        parse_peak_channel_hint(
                            props,
                            &[
                                "audio.peak",
                                "peak",
                                "monitor.peak",
                                "monitor.channel-peaks",
                            ],
                            0,
                        )
                    })
                    .or_else(|| {
                        parse_peak_channel_hint(
                            props,
                            &[
                                "monitor.channel-volumes",
                                "channelmix.volumes",
                                "audio.channel.volumes",
                            ],
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

    let mut nodes = match Rc::try_unwrap(nodes) {
        Ok(nodes) => nodes.into_inner(),
        Err(_) => return Err("failed to unwrap collected nodes".into()),
    };

    if !probed_source_ids.is_empty() {
        for node in &mut nodes {
            if !probed_source_ids.contains(&node.id) {
                continue;
            }

            let peaks_missing = node.peak_left_hint.unwrap_or(0.0) <= 0.0
                && node.peak_right_hint.unwrap_or(0.0) <= 0.0;

            if !peaks_missing {
                continue;
            }

            if let Some((left, right)) = probe_source_levels(node.id, node.channels_hint) {
                node.peak_left_hint = Some(left);
                node.peak_right_hint = Some(right);
            }
        }
    }

    debug!("node_discovery: collected {} nodes", nodes.len());

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
