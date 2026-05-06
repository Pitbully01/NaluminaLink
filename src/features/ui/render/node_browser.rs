use eframe::egui;

use super::cards::NodeChoice;
use super::super::components::section_header;
use super::super::{InputChannel, NaluminaApp};

impl NaluminaApp {
    pub(in crate::features::ui) fn node_choices(&self) -> Vec<NodeChoice> {
        self.nodes
            .iter()
            .map(|node| NodeChoice {
                id: node.id,
                name: node.name.clone(),
            })
            .collect()
    }

    pub(in crate::features::ui) fn render_node_browser(&self, ui: &mut egui::Ui) {
        section_header(
            ui,
            self.i18n.text("ui.section.node_browser"),
            self.i18n.text("ui.section.node_browser_subtitle"),
        );

        egui::ScrollArea::vertical()
            .id_source("node_browser")
            .max_height(210.0)
            .show(ui, |ui| {
                for node in &self.nodes {
                    ui.horizontal_wrapped(|ui| {
                        ui.strong(format!("#{}", node.id));
                        ui.label(&node.name);
                        if let Some(ch) = node.channels_hint {
                            ui.label(
                                egui::RichText::new(
                                    self.i18n.text_with(
                                        "ui.node.channels",
                                        &[("count", ch.to_string())],
                                    ),
                                )
                                .small(),
                            );
                        }
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

                ui.label(
                    egui::RichText::new(self.i18n.text_with(
                        "ui.nodes.visible_count",
                        &[
                            ("shown", self.nodes.len().to_string()),
                            ("total", self.nodes.len().to_string()),
                        ],
                    ))
                    .small(),
                );
            });
    }

    pub(in crate::features::ui) fn source_live_levels(&self, source_node_id: Option<u32>) -> (f32, f32) {
        let Some(node_id) = source_node_id else {
            return (0.0, 0.0);
        };

            let Some(node) = self.nodes.iter().find(|node| node.id == node_id) else {
            return (0.0, 0.0);
        };

        if let Some(snapshot) = self.live_meter_store.reading(node_id) {
            return (
                snapshot.current.left.clamp(0.0, 1.0),
                snapshot.current.right.clamp(0.0, 1.0),
            );
        }

        let fallback = node.volume_hint.unwrap_or(0.0).clamp(0.0, 1.0);
        let left = node.peak_left_hint.unwrap_or(fallback).clamp(0.0, 1.0);
        let right = node.peak_right_hint.unwrap_or(left).clamp(0.0, 1.0);
        (left, right)
    }

    pub(in crate::features::ui) fn source_live_level(&self, source_node_id: Option<u32>) -> f32 {
        let (left, right) = self.source_live_levels(source_node_id);
            return left.max(right);
    }

    pub(in crate::features::ui) fn source_peak_level(&self, source_node_id: Option<u32>) -> f32 {
        let Some(node_id) = source_node_id else {
            return 0.0;
        };

            if let Some(snapshot) = self.live_meter_store.reading(node_id) {
            return snapshot.peak.left.max(snapshot.peak.right).clamp(0.0, 1.0);
        }

        self.source_live_level(source_node_id)
    }

    pub(in crate::features::ui) fn source_label(&self, source_node_id: Option<u32>) -> String {
        let Some(node_id) = source_node_id else {
            return self.i18n.text("ui.device.unassigned");
        };

        self.nodes
            .iter()
            .find(|node| node.id == node_id)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| self.i18n.text("ui.device.unassigned"))
    }

    pub(in crate::features::ui) fn visible_input_channels(&self) -> Vec<InputChannel> {
        let filter = self.node_filter.trim().to_lowercase();

        self.input_channels
            .iter()
            .filter(|channel| {
                if filter.is_empty() {
                    return true;
                }

                let id_match = channel.id.to_string().contains(&filter);
                let name_match = channel.name.to_lowercase().contains(&filter);
                let source_match = self
                    .source_label(channel.source_node_id)
                    .to_lowercase()
                    .contains(&filter);

                id_match || name_match || source_match
            })
            .take(self.visible_channel_limit)
            .cloned()
            .collect()
    }
}
