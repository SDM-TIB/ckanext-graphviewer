use crate::graph_processor::{Edge, Node};
use eframe::egui;
use std::collections::{HashMap, VecDeque};

pub enum HierarchyType {
    Radial,
    Horizontal,
    Vertical,
}

pub fn apply_layout(
    nodes: &mut [Node],
    edges: &[Edge],
    visible_indices: &[usize],
    root_node_id: &mut Option<String>,
    layout_type: HierarchyType,
) {
    // Build adjacency list for visible connections
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for e in edges.iter().filter(|e| e.visible) {
        adj.entry(e.source).or_default().push(e.target);
        adj.entry(e.target).or_default().push(e.source);
    }

    // Identify root nodes
    let mut roots = Vec::new();

    // 1. Try to use the saved root node
    if let &mut Some(ref root_id) = root_node_id {
        if let Some(&idx) =
            visible_indices.iter().find(|&&i| nodes[i].id == *root_id)
        {
            roots.push(idx);
        }
    }

    // 2. If no saved root (or it's hidden), look for a node with `is_root`
    if roots.is_empty() {
        for &idx in visible_indices {
            if nodes[idx].is_root {
                roots.push(idx);
            }
        }
        if let Some(&first_root) = roots.first() {
            *root_node_id = Some(nodes[first_root].id.clone());
        }
    }

    // 3. Fallback to node with highest degree
    if roots.is_empty() {
        let max_deg_node = visible_indices
            .iter()
            .copied()
            .max_by_key(|&idx| {
                adj.get(&idx).map_or(0, |neighbors| neighbors.len())
            })
            .unwrap_or(visible_indices[0]);
        roots.push(max_deg_node);
        *root_node_id = Some(nodes[max_deg_node].id.clone());
    }

    // BFS to assign hierarchy levels
    let mut levels: HashMap<usize, usize> = HashMap::new();
    let mut queue = VecDeque::new();

    for &r in &roots {
        queue.push_back((r, 0));
        levels.insert(r, 0);
    }

    while let Some((curr, lvl)) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&curr) {
            for &nxt in neighbors {
                if !levels.contains_key(&nxt) {
                    levels.insert(nxt, lvl + 1);
                    queue.push_back((nxt, lvl + 1));
                }
            }
        }
    }

    // Assign isolated components to level 0
    for &idx in visible_indices {
        if !levels.contains_key(&idx) {
            levels.insert(idx, 0);
        }
    }

    // Calculate node distribution per level
    let mut level_counts: HashMap<usize, usize> = HashMap::new();
    for &lvl in levels.values() {
        *level_counts.entry(lvl).or_default() += 1;
    }

    let mut current_in_level: HashMap<usize, usize> = HashMap::new();

    // sort by node level
    let mut sorted_indices = visible_indices.to_vec();
    sorted_indices.sort_by(|&a, &b| {
        nodes[a]
            .label
            .to_lowercase()
            .cmp(&nodes[b].label.to_lowercase())
    });

    // Apply calculated positions
    for &idx in &sorted_indices {
        let lvl = levels[&idx];
        let pos_in_lvl = current_in_level.entry(lvl).or_default();
        let count = level_counts[&lvl];

        let spacing_x = 300.0;
        let spacing_y = 75.0;

        match layout_type {
            HierarchyType::Horizontal => {
                let offset_y = *pos_in_lvl as f32 - (count as f32 - 1.0) / 2.0;
                nodes[idx].pos =
                    egui::pos2(lvl as f32 * spacing_x, offset_y * spacing_y);
            }
            HierarchyType::Vertical => {
                let offset_x = *pos_in_lvl as f32 - (count as f32 - 1.0) / 2.0;
                nodes[idx].pos =
                    egui::pos2(offset_x * spacing_y, lvl as f32 * spacing_x);
            }
            HierarchyType::Radial => {
                if lvl == 0 {
                    let root_offset = (*pos_in_lvl as f32
                        - (count as f32 - 1.0) / 2.0)
                        * 80.0;
                    nodes[idx].pos = egui::pos2(root_offset, 0.0);
                } else {
                    let radius = lvl as f32 * 200.0;
                    let angle = (*pos_in_lvl as f32 / count as f32)
                        * std::f32::consts::TAU;
                    nodes[idx].pos =
                        egui::pos2(angle.cos() * radius, angle.sin() * radius);
                }
            }
        }
        *pos_in_lvl += 1;
    }
}
