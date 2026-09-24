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

        for t in parsed.raw_triples {
            raw_triples.push(crate::parser::RawTriple {
                subject: t.subject,
                predicate: t.predicate,
                object: t.object,
                is_object_literal: t.is_object_literal,
            });
        }

        raw_triples.sort();
        raw_triples.dedup();

        // build the topology
        let (mut generated_nodes, mut generated_edges) =
            crate::graph_processor::build_ui_graph(raw_triples.clone(), None);

        // reset everything to hidden
        for n in &mut generated_nodes {
            n.visible = false;
            n.expanded = false;
            n.api_fetched = false;
            n.has_more_to_fetch = true;
        }
        for e in &mut generated_edges {
            e.visible = false;
        }

        // apply visability
        let mut json_node_map = std::collections::HashMap::new();
        for n in parsed.nodes {
            json_node_map.insert(n.id.clone(), n);
        }

        for n in &mut generated_nodes {
            if let Some(saved) = json_node_map.get(&n.id) {
                n.pos = eframe::egui::Pos2::new(saved.x, saved.y);
                n.original_pos = eframe::egui::Pos2::new(saved.x, saved.y);
                n.visible = true;
                n.expanded = true;
            }
        }

        // edge visability
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
