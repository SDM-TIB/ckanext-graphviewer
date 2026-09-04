use crate::{AppState, graph_processor};
use eframe::egui;
use std::sync::{Arc, Mutex};
use log::debug;

fn process_graph_update(
    ctx: egui::Context,
    state: Arc<Mutex<AppState>>,
    clicked_node_id: String,
    json_text: &str,
    pagination_limit: Option<usize>,
) {
    let new_triples = crate::parser::parse_dynamic_api_json(json_text);

    let returned_count = serde_json::from_str::<serde_json::Value>(json_text)
        .ok()
        .and_then(|v| {
            v.get("results").and_then(|r| {
                r.as_object().map(|o| o.len()).or_else(|| r.as_array().map(|a| a.len()))
            })
        })
        .unwrap_or(0);

    let mut state_lock = state.lock().unwrap();

    if let AppState::Ready { raw_triples, nodes, edges, .. } = &mut *state_lock {
        // create a snapshot of the state
        let mut old_nodes = std::collections::HashMap::new();
        for n in nodes.iter() {
            old_nodes.insert(n.id.clone(), n.clone());
        }

        let mut old_edges_vis = std::collections::HashSet::new();
        for e in edges.iter() {
            if e.visible {
                let s_id = nodes[e.source].id.clone();
                let t_id = nodes[e.target].id.clone();
                old_edges_vis.insert((s_id, t_id));
            }
        }

        // rebuild with new triples
        raw_triples.extend(new_triples);
        let (mut new_nodes, mut new_edges) = graph_processor::build_ui_graph(raw_triples.clone(), None);

        // edge normalization
        for edge in &mut new_edges {
            let source_type = &new_nodes[edge.source].rdf_type;
            let target_type = &new_nodes[edge.target].rdf_type;

            let source_is_dataset = source_type.contains("Dataset") || source_type.contains("DataService");
            let target_is_author = target_type.contains("Author");

            if source_is_dataset && target_is_author {
                std::mem::swap(&mut edge.source, &mut edge.target);
                if let Some(rev) = edge.reverse_label.clone() {
                    edge.reverse_label = Some(edge.label.clone());
                    edge.label = rev;
                }
            }
        }

        // map connections to the clicked node
        let mut connected_nodes = std::collections::HashSet::new();
        for edge in new_edges.iter() {
            let s_id = &new_nodes[edge.source].id;
            let t_id = &new_nodes[edge.target].id;
            if s_id == &clicked_node_id {
                connected_nodes.insert(t_id.clone());
            } else if t_id == &clicked_node_id {
                connected_nodes.insert(s_id.clone());
            }
        }

        // visibility and layout tracking
        let clicked_pos = old_nodes.get(&clicked_node_id).map(|n| n.pos).unwrap_or(egui::Pos2::ZERO);
        let mut nodes_to_layout = Vec::new();

        for (i, n) in new_nodes.iter_mut().enumerate() {
            if let Some(old_n) = old_nodes.get(&n.id) {
                // save old state
                n.pos = old_n.pos;
                n.original_pos = old_n.original_pos;
                n.visible = old_n.visible;
                n.api_fetched = old_n.api_fetched;
                n.fetch_offset = old_n.fetch_offset;
                n.has_more_to_fetch = old_n.has_more_to_fetch;

                if n.id == clicked_node_id {
                    n.expanded = true;

                    if let Some(limit) = pagination_limit {
                        if returned_count < limit {
                            n.has_more_to_fetch = false;
                            n.api_fetched = true;
                        } else {
                            n.fetch_offset += limit;
                            n.has_more_to_fetch = true;
                            n.api_fetched = false;
                        }
                    } else {
                        n.has_more_to_fetch = false;
                    }
                } else if connected_nodes.contains(&n.id) && !n.visible {
                    n.visible = true;
                    nodes_to_layout.push(i);
                } else {
                    n.expanded = old_n.expanded;
                }
            } else {
                // init new node
                n.api_fetched = false;
                n.fetch_offset = 0;
                n.has_more_to_fetch = true;

                if connected_nodes.contains(&n.id) {
                    n.visible = true;
                    n.expanded = false;
                    nodes_to_layout.push(i);
                } else {
                    n.visible = false;
                    n.expanded = false;
                }
            }
        }

        // position new nodes radially
        let total_layout = nodes_to_layout.len();
        if total_layout > 0 {
            let mut angle: f32 = 0.0;
            let angle_step = std::f32::consts::TAU / (total_layout as f32);
            let spawn_radius = 240.0;

            for idx in nodes_to_layout {
                let target_pos = clicked_pos + egui::vec2(angle.cos() * spawn_radius, angle.sin() * spawn_radius);
                angle += angle_step;

                new_nodes[idx].pos = target_pos;
                new_nodes[idx].original_pos = target_pos;
            }
        }

        // restore edge visibility
        for edge in &mut new_edges {
            let s_id = &new_nodes[edge.source].id;
            let t_id = &new_nodes[edge.target].id;

            if old_edges_vis.contains(&(s_id.clone(), t_id.clone())) || old_edges_vis.contains(&(t_id.clone(), s_id.clone())) {
                edge.visible = true;
            } else if s_id == &clicked_node_id || t_id == &clicked_node_id {
                edge.visible = true;
            } else {
                edge.visible = false;
            }
        }

        *nodes = new_nodes;
        *edges = new_edges;
    }

    ctx.request_repaint();
}

// fetch keyword
pub fn fetch_keyword_information(
    ctx: egui::Context,
    state: Arc<Mutex<AppState>>,
    clicked_node_id: String,
    keyword_id: String,
    offset: usize,
    limit: usize,
    api_url: &str,
) {
    let url = format!(
        "{}/get_dataset_information_by_keyword?keyword={}&limit={}&offset={}",
        api_url,
        urlencoding::encode(&keyword_id),
        limit,
        offset,
    );
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        if let Ok(res) = response {
            if let Some(text) = res.text() {
                process_graph_update(ctx, state, clicked_node_id, &text, Some(limit));
            }
        } else {
            process_graph_update(ctx, state, clicked_node_id, "{\"results\":{}}", Some(limit));
        }
    });
}

// fetch author
pub fn fetch_author_information(
    ctx: egui::Context,
    state: Arc<Mutex<AppState>>,
    clicked_node_id: String,
    author_id: String,
    offset: usize,
    limit: usize,
    api_url: &str,
) {
    let url = format!(
        "{}/get_dataset_information_by_author_ldm_id?author_ldm_id={}&limit={}&offset={}",
        api_url,
        urlencoding::encode(&author_id),
        limit,
        offset,
    );
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        if let Ok(res) = response {
            if let Some(text) = res.text() {
                process_graph_update(ctx, state, clicked_node_id, &text, Some(limit));
            }
        } else {
            process_graph_update(ctx, state, clicked_node_id, "{\"results\":{}}", Some(limit));
        }
    });
}

// fetch dataset
pub fn fetch_dataset_information(
    ctx: egui::Context,
    state: Arc<Mutex<AppState>>,
    clicked_node_id: String,
    dataset_id: String,
    offset: usize,
    limit: usize,
    api_url: &str,
) {
    let url = format!(
        "{}/get_dataset_information_by_dataset_ldm_id?dataset_ldm_id={}&limit={}&offset={}",
        api_url,
        urlencoding::encode(&dataset_id),
        limit,
        offset,
    );
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        if let Ok(res) = response {
            if let Some(text) = res.text() {
                process_graph_update(ctx, state, clicked_node_id, &text, Some(limit));
            }
        } else {
            process_graph_update(ctx, state, clicked_node_id, "{\"results\":{}}", Some(limit));
        }
    });
}

// fetch publisher
pub fn fetch_publisher_information(
    ctx: egui::Context,
    state: Arc<Mutex<AppState>>,
    clicked_node_id: String,
    publisher_id: String,
    offset: usize,
    limit: usize,
    api_url: &str,
) {
    let url = format!(
        "{}/get_dataset_information_by_publisher?publisher_id={}&limit={}&offset={}",
        api_url,
        urlencoding::encode(&publisher_id),
        limit,
        offset,
    );
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        if let Ok(res) = response {
            if let Some(text) = res.text() {
                process_graph_update(ctx, state, clicked_node_id, &text, Some(limit));
            }
        } else {
            process_graph_update(ctx, state, clicked_node_id, "{\"results\":{}}", Some(limit));
        }
    });
}
