use crate::graph_processor::Node;
use eframe::egui;

pub fn apply_layout(nodes: &mut [Node], visible_indices: &[usize]) {
    let n = visible_indices.len();
    let radius = (n as f32 * 25.0).max(150.0);

    for (i, &idx) in visible_indices.iter().enumerate() {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
        nodes[idx].pos = egui::pos2(angle.cos() * radius, angle.sin() * radius);
    }
}
