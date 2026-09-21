use crate::{AppState, GraphSnapshot};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[derive(Deserialize)]
struct ImportedNode {
    id: String,
    #[serde(default)]
    rdf_type: String,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    properties: Vec<ImportedProperty>,
}

#[derive(Deserialize)]
struct ImportedProperty {
    predicate: String,
    object: String,
}

#[derive(Deserialize)]
struct ImportedEdge {
    source: String,
    target: String,
    label: String,
    #[serde(default)]
    reverse_label: Option<String>,
    #[serde(default)]
    bidirectional: bool,
}

#[derive(Deserialize)]
struct ImportedTriple {
    subject: String,
    predicate: String,
    object: String,
    #[serde(default)]
    is_object_literal: bool,
}

#[derive(Deserialize)]
struct ImportedGraph {
    #[serde(default)]
    nodes: Vec<ImportedNode>,
    #[serde(default)]
    edges: Vec<ImportedEdge>,
    #[serde(default)]
    raw_triples: Vec<ImportedTriple>,
}

pub fn trigger_json_import(
    state: Arc<Mutex<AppState>>,
    ctx: eframe::egui::Context,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(file) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            if let Ok(content) = std::fs::read_to_string(file) {
                process_json_content(&content, state);
                ctx.request_repaint();
            } else {
                log::error!("Failed to read selected JSON file.");
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
                .await
            {
                let bytes = file.read().await;
                if let Ok(content) = String::from_utf8(bytes) {
                    process_json_content(&content, state);
                    ctx.request_repaint();
                } else {
                    log::error!("Failed to parse JSON file as UTF-8.");
                }
            }
        });
    }
}

fn process_json_content(content: &str, state: Arc<Mutex<AppState>>) {
    if let Ok(parsed) = serde_json::from_str::<ImportedGraph>(content) {
        let mut raw_triples = Vec::new();

        // 1. Reconstruct raw triples
        if parsed.raw_triples.is_empty() {
            // Fallback: If raw_triples are missing (old JSON format), meticulously reconstruct
            // them from the nodes, properties, and edges arrays so the graph doesn't break.
            for n in &parsed.nodes {
                let subj = format!("<{}>", n.id);

                // Reconstruct RDF Types
                if !n.rdf_type.is_empty() {
                    for t in n.rdf_type.split(", ") {
                        raw_triples.push(crate::parser::RawTriple {
                            subject: subj.clone(),
                            predicate: format!(
                                "<{}>",
                                crate::constants::RDF_TYPE
                            ),
                            object: format!("<{}>", t),
                            is_object_literal: false,
                        });
                    }
                }

                // Reconstruct Literal & URI Properties
                for p in &n.properties {
                    let is_lit = p.object.starts_with('"');
                    let obj = if is_lit {
                        p.object.clone()
                    } else {
                        format!("<{}>", p.object)
                    };
                    raw_triples.push(crate::parser::RawTriple {
                        subject: subj.clone(),
                        predicate: format!("<{}>", p.predicate), // Clean label will be parsed back accurately
                        object: obj,
                        is_object_literal: is_lit,
                    });
                }
            }

            // Reconstruct relationships
            for e in &parsed.edges {
                for label in e.label.split(", ") {
                    raw_triples.push(crate::parser::RawTriple {
                        subject: format!("<{}>", e.source),
                        predicate: format!("<{}>", label),
                        object: format!("<{}>", e.target),
                        is_object_literal: false,
                    });
                }
                if let Some(rev) = &e.reverse_label {
                    for label in rev.split(", ") {
                        raw_triples.push(crate::parser::RawTriple {
                            subject: format!("<{}>", e.target),
                            predicate: format!("<{}>", label),
                            object: format!("<{}>", e.source),
                            is_object_literal: false,
                        });
                    }
                }
            }
        } else {
            // Direct load for new JSON exports
            for t in parsed.raw_triples {
                raw_triples.push(crate::parser::RawTriple {
                    subject: t.subject,
                    predicate: t.predicate,
                    object: t.object,
                    is_object_literal: t.is_object_literal,
                });
            }
        }

        raw_triples.sort();
        raw_triples.dedup();

        // 2. Build the topology perfectly from the triples using the single source of truth
        let (mut generated_nodes, mut generated_edges) =
            crate::graph_processor::build_ui_graph(raw_triples.clone(), None);

        // 3. Overlay the visual state saved in the JSON
        // Reset all generated elements to hidden/unexplored first
        for n in &mut generated_nodes {
            n.visible = false;
            n.expanded = false;
            n.api_fetched = false; // Ensures clicking '+' triggers an API fetch
            n.has_more_to_fetch = true;
        }
        for e in &mut generated_edges {
            e.visible = false;
        }

        // Apply saved positions and visibility for nodes
        let mut json_node_map = std::collections::HashMap::new();
        for n in parsed.nodes {
            // Clone the ID so the string isn't moved out of `n` before `n` is inserted
            json_node_map.insert(n.id.clone(), n);
        }

        for n in &mut generated_nodes {
            if let Some(saved) = json_node_map.get(&n.id) {
                n.pos = eframe::egui::Pos2::new(saved.x, saved.y);
                n.original_pos = eframe::egui::Pos2::new(saved.x, saved.y);
                n.visible = true;
                n.expanded = true; // Was expanded when saved
            }
        }

        // Apply saved visibility for edges
        let mut json_edge_set = std::collections::HashSet::new();
        for e in parsed.edges {
            json_edge_set.insert((e.source.clone(), e.target.clone()));
            if e.bidirectional {
                json_edge_set.insert((e.target, e.source));
            }
        }

        for e in &mut generated_edges {
            let s_id = &generated_nodes[e.source].id;
            let t_id = &generated_nodes[e.target].id;

            if json_edge_set.contains(&(s_id.clone(), t_id.clone()))
                || json_edge_set.contains(&(t_id.clone(), s_id.clone()))
            {
                e.visible = true;
            }
        }

        let snapshot = GraphSnapshot::new(&generated_nodes, &generated_edges);

        if let Ok(mut lock) = state.lock() {
            *lock = AppState::Ready {
                nodes: generated_nodes,
                edges: generated_edges,
                raw_triples,
                init_snapshot: snapshot,
            };
        }
    } else {
        log::error!(
            "Failed to deserialize JSON content. Ensure it is a valid LDM JSON export."
        );
    }
}
