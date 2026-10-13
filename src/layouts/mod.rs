pub mod circle;
pub mod hierarchy;

use crate::GraphLayout;
use crate::graph_processor::{Edge, Node};

pub fn apply(
    layout: GraphLayout,
    nodes: &mut [Node],
    edges: &[Edge],
    root_node_id: &mut Option<String>,
) {
    let visible_indices: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.visible)
        .map(|(i, _)| i)
        .collect();

    if visible_indices.is_empty() {
        return;
    }

    match layout {
        GraphLayout::Circle => circle::apply_layout(nodes, &visible_indices),
        GraphLayout::Radial => hierarchy::apply_layout(
            nodes,
            edges,
            &visible_indices,
            root_node_id,
            hierarchy::HierarchyType::Radial,
        ),
        GraphLayout::HorizontalHierarchical => hierarchy::apply_layout(
            nodes,
            edges,
            &visible_indices,
            root_node_id,
            hierarchy::HierarchyType::Horizontal,
        ),
        GraphLayout::VerticalHierarchical => hierarchy::apply_layout(
            nodes,
            edges,
            &visible_indices,
            root_node_id,
            hierarchy::HierarchyType::Vertical,
        ),
    }
}
