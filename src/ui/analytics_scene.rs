use crate::App;
use crate::graph_processor::{Edge, Node};
use crate::parser::RawTriple;
use eframe::egui;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

impl App {
    fn render_stat_row(
        &self,
        ui: &mut egui::Ui,
        col1_width: f32,
        col2_width: f32,
        col3_width: f32,
        label: &str,
        total_str: &str,
        visible_str: &str,
    ) {
        // 1. column label
        ui.vertical(|ui| {
            ui.set_min_width(col1_width);
            ui.set_max_width(col1_width);

            ui.label(egui::RichText::new(label).color(self.ui.theme.text_fg));
        });

        // 2. column total count
        ui.vertical(|ui| {
            ui.set_min_width(col2_width);
            ui.set_max_width(col2_width);

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(total_str)
                            .color(self.ui.theme.text_fg),
                    ));
                },
            );
        });

        // 3. column visible count
        ui.vertical(|ui| {
            ui.set_min_width(col3_width);
            ui.set_max_width(col3_width);

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(visible_str)
                            .color(self.ui.theme.text_fg),
                    ));
                },
            );
        });

        ui.end_row();
    }

    pub fn render_analytics_scene(
        &mut self,
        ui: &mut egui::Ui,
        nodes: &[Node],
        edges: &[Edge],
        raw_triples: &[RawTriple],
    ) {
        // render analytics
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // calculate graph statistics
                let mut unique_subjects = std::collections::HashSet::new();
                let mut unique_predicates = std::collections::HashSet::new();
                let mut unique_objects = std::collections::HashSet::new();
                let mut unique_classes = std::collections::HashSet::new();
                let mut unique_instances = std::collections::HashSet::new();
                let mut object_properties = std::collections::HashSet::new();
                let mut datatype_properties = std::collections::HashSet::new();
                let mut namespaces = std::collections::HashSet::new();

                // new sets for visible metrics
                let mut vis_unique_subjects = std::collections::HashSet::new();
                let mut vis_unique_predicates =
                    std::collections::HashSet::new();
                let mut vis_unique_objects = std::collections::HashSet::new();
                let mut vis_unique_classes = std::collections::HashSet::new();
                let mut vis_unique_instances = std::collections::HashSet::new();
                let mut vis_object_properties =
                    std::collections::HashSet::new();
                let mut vis_datatype_properties =
                    std::collections::HashSet::new();
                let mut vis_namespaces = std::collections::HashSet::new();
                let mut visible_triples_count = 0;

                let rdf_type_uri =
                    "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";

                // Helper to rip namespaces out of URIs
                let extract_namespace = |uri: &str| -> Option<String> {
                    let clean = uri.trim_matches('<').trim_matches('>');
                    if let Some(idx) =
                        clean.rfind('#').or_else(|| clean.rfind('/'))
                    {
                        Some(clean[..=idx].to_string())
                    } else {
                        None
                    }
                };

                // lookup table for visible nodes
                let visible_node_ids: std::collections::HashSet<&str> = nodes
                    .iter()
                    .filter(|n| n.visible)
                    .map(|n| n.id.as_str())
                    .collect();

                for t in raw_triples.iter() {
                    unique_subjects.insert(&t.subject);
                    unique_predicates.insert(&t.predicate);
                    unique_objects.insert(&t.object);

                    // check ontology classes and instances
                    if t.predicate == rdf_type_uri {
                        unique_classes.insert(&t.object);
                        unique_instances.insert(&t.subject);
                    }

                    // datatype and object properties
                    if t.is_object_literal {
                        datatype_properties.insert(&t.predicate);
                    } else {
                        object_properties.insert(&t.predicate);
                    }

                    // namespaces
                    if let Some(ns) = extract_namespace(&t.subject) {
                        namespaces.insert(ns);
                    }
                    if let Some(ns) = extract_namespace(&t.predicate) {
                        namespaces.insert(ns);
                    }
                    if !t.is_object_literal {
                        if let Some(ns) = extract_namespace(&t.object) {
                            namespaces.insert(ns);
                        }
                    }

                    // TODO: implement literals
                    // visibility logic
                    let clean_subj =
                        t.subject.trim_matches(|c| c == '<' || c == '>');
                    let clean_obj =
                        t.object.trim_matches(|c| c == '<' || c == '>');

                    let subj_vis = visible_node_ids.contains(clean_subj)
                        || visible_node_ids.contains(t.subject.as_str());
                    let obj_vis = visible_node_ids.contains(clean_obj)
                        || visible_node_ids.contains(t.object.as_str());

                    let is_triple_visible = if t.predicate == rdf_type_uri {
                        subj_vis
                    } else {
                        subj_vis && obj_vis
                    };

                    if is_triple_visible {
                        visible_triples_count += 1;
                        vis_unique_subjects.insert(&t.subject);
                        vis_unique_predicates.insert(&t.predicate);
                        vis_unique_objects.insert(&t.object);

                        if t.predicate == rdf_type_uri {
                            vis_unique_classes.insert(&t.object);
                            vis_unique_instances.insert(&t.subject);
                        }

                        if t.is_object_literal {
                            vis_datatype_properties.insert(&t.predicate);
                        } else {
                            vis_object_properties.insert(&t.predicate);
                        }

                        if let Some(ns) = extract_namespace(&t.subject) {
                            vis_namespaces.insert(ns);
                        }
                        if let Some(ns) = extract_namespace(&t.predicate) {
                            vis_namespaces.insert(ns);
                        }
                        if !t.is_object_literal {
                            if let Some(ns) = extract_namespace(&t.object) {
                                vis_namespaces.insert(ns);
                            }
                        }
                    }
                }

                let total_triples = raw_triples.len();
                let total_nodes = nodes.len();
                let total_edges = edges.len();
                let visible_nodes = nodes.iter().filter(|n| n.visible).count();
                let visible_edges = edges.iter().filter(|e| e.visible).count();

                let uri_nodes =
                    nodes.iter().filter(|n| n.node_type == "NamedNode").count();
                let visible_uri_nodes = nodes
                    .iter()
                    .filter(|n| n.node_type == "NamedNode" && n.visible)
                    .count();

                let literal_nodes =
                    nodes.iter().filter(|n| n.node_type == "Attribute").count();
                let visible_literal_nodes = nodes
                    .iter()
                    .filter(|n| n.node_type == "Attribute" && n.visible)
                    .count();

                let blank_nodes =
                    nodes.iter().filter(|n| n.node_type == "BlankNode").count();
                let visible_blank_nodes = nodes
                    .iter()
                    .filter(|n| n.node_type == "BlankNode" && n.visible)
                    .count();

                // calculate node types and their visible counts
                let mut type_counts = std::collections::HashMap::new();
                let mut visible_type_counts = std::collections::HashMap::new();
                for n in nodes.iter() {
                    if n.rdf_type.is_empty() {
                        *type_counts
                            .entry("Untyped Node".to_string())
                            .or_insert(0) += 1;
                        if n.visible {
                            *visible_type_counts
                                .entry("Untyped Node".to_string())
                                .or_insert(0) += 1;
                        }
                    } else {
                        for single_type in n.rdf_type.split(", ") {
                            let display_name = single_type
                                .split('#')
                                .last()
                                .unwrap_or(single_type);
                            let display_name = display_name
                                .split('/')
                                .last()
                                .unwrap_or(display_name)
                                .to_string();
                            *type_counts
                                .entry(display_name.clone())
                                .or_insert(0) += 1;
                            if n.visible {
                                *visible_type_counts
                                    .entry(display_name)
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                }

                let mut sorted_types: Vec<(String, i32, i32)> = Vec::new();
                for (name, total) in type_counts {
                    let visible = *visible_type_counts.get(&name).unwrap_or(&0);
                    sorted_types.push((name, total, visible));
                }
                sorted_types.sort_by(|a, b| {
                    a.0.to_lowercase().cmp(&b.0.to_lowercase())
                });

                // draw the 2x2 grid
                // here lies magic for maring reasons
                let tile_height =
                    ((ui.available_height() - 36.0) / 2.0).floor();
                let tile_width = ((ui.available_width() - 35.0) / 2.0).floor();

                // split tile width into 3 columns
                let content_width = tile_width - 24.0;
                let col1_width = content_width * 0.60;
                let col2_width = content_width * 0.20;
                let col3_width = content_width * 0.20;

                // make upper margin 6 pixel
                ui.add_space(1.0);

                // row 1
                ui.columns(2, |cols| {
                    // left
                    cols[0].group(|ui| {
                        ui.set_min_height(tile_height);
                        ui.set_min_width(tile_width);
                        ui.heading(
                            egui::RichText::new("Triple Composition")
                                .color(self.ui.theme.text_fg),
                        );
                        ui.add_space(10.0);

                        egui::ScrollArea::vertical()
                            .id_salt("triple_composition_scroll")
                            .show(ui, |ui| {
                                egui::Grid::new("triple_grid")
                                    .num_columns(3)
                                    .show(ui, |ui| {
                                        self.render_stat_row(
                                            ui, col1_width, col2_width,
                                            col3_width, "Metric", "Total",
                                            "Visible",
                                        );
                                        ui.end_row();

                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Total Triples",
                                            &total_triples.to_string(),
                                            &visible_triples_count.to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Total Nodes",
                                            &total_nodes.to_string(),
                                            &visible_nodes.to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Total Edges",
                                            &total_edges.to_string(),
                                            &visible_edges.to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "URI Nodes",
                                            &uri_nodes.to_string(),
                                            &visible_uri_nodes.to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Literal Nodes",
                                            &literal_nodes.to_string(),
                                            &visible_literal_nodes.to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Blank Nodes",
                                            &blank_nodes.to_string(),
                                            &visible_blank_nodes.to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Unique Subjects",
                                            &unique_subjects.len().to_string(),
                                            &vis_unique_subjects
                                                .len()
                                                .to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Unique Predicates",
                                            &unique_predicates
                                                .len()
                                                .to_string(),
                                            &vis_unique_predicates
                                                .len()
                                                .to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Unique Objects",
                                            &unique_objects.len().to_string(),
                                            &vis_unique_objects
                                                .len()
                                                .to_string(),
                                        );
                                    });
                            });
                    });

                    // right
                    cols[1].group(|ui| {
                        ui.set_min_height(tile_height);
                        ui.set_min_width(tile_width);
                        ui.heading(
                            egui::RichText::new("Ontology Statistics")
                                .color(self.ui.theme.text_fg),
                        );
                        ui.add_space(10.0);

                        egui::ScrollArea::vertical()
                            .id_salt("ontology_statistics_scroll")
                            .show(ui, |ui| {
                                egui::Grid::new("ontology_grid")
                                    .num_columns(3)
                                    .show(ui, |ui| {
                                        self.render_stat_row(
                                            ui, col1_width, col2_width,
                                            col3_width, "Metric", "Total",
                                            "Visible",
                                        );
                                        ui.end_row();

                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Classes",
                                            &unique_classes.len().to_string(),
                                            &vis_unique_classes
                                                .len()
                                                .to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Instances",
                                            &unique_instances.len().to_string(),
                                            &vis_unique_instances
                                                .len()
                                                .to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Object Properties",
                                            &object_properties
                                                .len()
                                                .to_string(),
                                            &vis_object_properties
                                                .len()
                                                .to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Datatype Properties",
                                            &datatype_properties
                                                .len()
                                                .to_string(),
                                            &vis_datatype_properties
                                                .len()
                                                .to_string(),
                                        );
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Namespaces",
                                            &namespaces.len().to_string(),
                                            &vis_namespaces.len().to_string(),
                                        );
                                    });
                            });
                    });
                });

                ui.add_space(5.0);

                // row 2
                ui.columns(2, |cols| {
                    // left
                    cols[0].group(|ui| {
                        ui.set_min_height(tile_height);
                        ui.set_min_width(tile_width);
                        ui.heading(
                            egui::RichText::new("Node Types Breakdown")
                                .color(self.ui.theme.text_fg),
                        );
                        ui.add_space(10.0);

                        egui::ScrollArea::vertical()
                            .id_salt("types_breakdown_scroll")
                            .show(ui, |ui| {
                                egui::Grid::new("analytics_grid")
                                    .num_columns(3)
                                    .show(ui, |ui| {
                                        self.render_stat_row(
                                            ui,
                                            col1_width,
                                            col2_width,
                                            col3_width,
                                            "Type Name",
                                            "Total",
                                            "Visible",
                                        );
                                        ui.end_row();

                                        for (
                                            display_name,
                                            total_count,
                                            visible_count,
                                        ) in sorted_types
                                        {
                                            self.render_stat_row(
                                                ui,
                                                col1_width,
                                                col2_width,
                                                col3_width,
                                                &display_name,
                                                &total_count.to_string(),
                                                &visible_count.to_string(),
                                            );
                                        }
                                    });
                            });
                    });

                    // right TODO: come up with a cool statistic for this
                    cols[1].group(|ui| {
                        ui.set_min_height(tile_height);
                        ui.set_min_width(tile_width);
                    });
                });
            });
    }
}
