use crate::graph_processor::{Edge, Node};
use crate::{App, GraphSnapshot};
use eframe::egui;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

impl App {
    pub fn render_graph_scene(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        nodes: &mut [Node],
        edges: &mut [Edge],
        init_snapshot: &mut GraphSnapshot,
    ) {
        // render graph
        ui.add_space(1.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Graph Controls:").color(self.ui.theme.text_fg));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let reset_view_button =
                    egui::Button::new(egui::RichText::new("Reset View").color(self.ui.theme.text_fg)).fill(self.ui.theme.button_bg);
                if ui
                    .add(reset_view_button)
                    .on_hover_text("Reset the graph view to the initial snapshot")
                    .clicked()
                {
                    self.ui.zoom = 1.0;
                    self.ui.pan = egui::vec2(0.0, 0.0);
                    self.ui.selected_node = None;

                    for node in nodes.iter_mut() {
                        if let Some(&pos) = init_snapshot.node_positions.get(&node.id) {
                            node.pos = pos;
                        } else {
                            node.pos = node.original_pos;
                        }
                        node.visible = init_snapshot.visible_nodes.contains(&node.id);
                        node.expanded = init_snapshot.expanded_nodes.contains(&node.id);
                    }

                    for edge in edges.iter_mut() {
                        let s_id = &nodes[edge.source].id;
                        let t_id = &nodes[edge.target].id;

                        edge.visible = init_snapshot.visible_edges.contains(&(s_id.clone(), t_id.clone()))
                            || init_snapshot.visible_edges.contains(&(t_id.clone(), s_id.clone()));
                    }

                    *init_snapshot = GraphSnapshot::new(nodes, edges);
                }
            });
        });
        ui.add_space(3.0);

        let draw_node_details = |ui: &mut egui::Ui, node: &Node| {
            egui::Grid::new(format!("tooltip_grid_{}", node.id))
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    let display_id = if node.node_type == "Attribute" { &node.label } else { &node.id };

                    if !display_id.is_empty() {
                        ui.strong("ID:");
                        if display_id.len() > 60 {
                            egui::ScrollArea::vertical()
                                .id_salt(format!("scroll_id_{}", node.id))
                                .max_height(60.0)
                                .min_scrolled_height(0.0)
                                .show(ui, |ui| {
                                    ui.label(display_id);
                                });
                        } else {
                            ui.label(display_id);
                        }
                        ui.end_row();
                    }

                    let mut seen_props = std::collections::HashSet::new();

                    for (i, (key, value)) in node.properties.iter().enumerate() {
                        if !seen_props.insert((key.clone(), value.clone())) {
                            continue;
                        }

                        let display_key = {
                            let mut c = key.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            }
                        };

                        ui.strong(format!("{}:", display_key));

                        if value.len() > 60 {
                            egui::ScrollArea::vertical()
                                .id_salt(format!("scroll_prop_{}_{}", node.id, i))
                                .max_height(100.0)
                                .min_scrolled_height(0.0)
                                .show(ui, |ui| {
                                    ui.label(value);
                                });
                        } else {
                            ui.label(value);
                        }
                        ui.end_row();
                    }
                });
        };

        // define a single frame that houses the color config for both legend and controls
        let combined_outline = egui::Frame::window(&ui.ctx().global_style())
            .fill(self.ui.theme.button_bg)
            .inner_margin(3.0)
            .corner_radius(5.0)
            .stroke(egui::Stroke::new(2.0_f32, self.ui.theme.master_bg));

        // render combined window without the outer title bar
        egui::Window::new("legend_and_controls_window")
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 101.0))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .frame(combined_outline)
            .show(ui.ctx(), |ui| {
                ui.style_mut().visuals.override_text_color = Some(self.ui.theme.text_fg);

                // Define the inner style for the content of the collapsing headers
                let inner_frame = egui::Frame::NONE
                    .inner_margin(5.0)
                    .corner_radius(5.0)
                    .stroke(egui::Stroke::new(1.0_f32, self.ui.theme.edge_fg))
                    .fill(self.ui.theme.master_bg);

                // Legend section
                egui::CollapsingHeader::new(egui::RichText::new("Legend").color(self.ui.theme.text_fg))
                    .default_open(false)
                    .show(ui, |ui| {
                        inner_frame.show(ui, |ui| {
                            egui::Grid::new("legend_grid").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
                                for (uri, colors) in &self.ui.theme.node_map {
                                    let display_name = uri.split('#').last().unwrap_or(uri);
                                    let display_name = display_name.split('/').last().unwrap_or(display_name);

                                    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                                    ui.painter().circle_filled(rect.center(), 6.0, colors.normal);

                                    ui.label(display_name);
                                    ui.end_row();
                                }

                                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                                ui.painter().circle_filled(rect.center(), 6.0, self.ui.theme.default_node.normal);
                                ui.label("Other");
                                ui.end_row();
                            });
                        });
                    });

                // Controls section
                egui::CollapsingHeader::new(egui::RichText::new("Controls").color(self.ui.theme.text_fg))
                    .default_open(false)
                    .show(ui, |ui| {
                        inner_frame.show(ui, |ui| {
                            egui::Grid::new("controls_window_grid")
                                .num_columns(2)
                                .spacing([20.0, 8.0])
                                .show(ui, |ui| {
                                    ui.strong("Move Camera:"); ui.label("Left-click and drag background");
                                    ui.end_row();
                                    ui.strong("Zoom Camera:"); ui.label("Scroll wheel or pinch");
                                    ui.end_row();
                                    ui.strong("Move Node:"); ui.label("Left-click and drag a node");
                                    ui.end_row();
                                    ui.strong("Open Infobox of a node:"); ui.label("Single left-click a node");
                                    ui.end_row();
                                    ui.strong("Expand / Collaps a node:"); ui.label("Double left-click a node");
                                    ui.end_row();
                                    ui.strong("Open Context Menu of a node:"); ui.label("Right-click a node");
                                    ui.end_row();
                                });
                        });
                    });
            });

        let background_rect = ui.available_rect_before_wrap();
        let area_to_fill = ui.available_rect_before_wrap();

        self.ui.canvas_rect = Some(area_to_fill);

        let screen_center = area_to_fill.center().to_vec2();

        let background_response = ui.interact(background_rect, ui.id().with("background"), egui::Sense::click_and_drag());
        if background_response.dragged() {
            self.ui.pan += background_response.drag_delta();
        }
        if background_response.clicked() {
            self.ui.selected_node = None;
        }

        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        let pinch_zoom = ui.input(|i| i.zoom_delta());

        let mut zoom_multiplier = pinch_zoom;
        if scroll_y != 0.0 {
            zoom_multiplier *= (scroll_y * 0.005).exp();
        }

        if zoom_multiplier != 1.0 {
            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                let pointer_vec = pointer_pos.to_vec2();
                let graph_pos = (pointer_vec - screen_center - self.ui.pan) / self.ui.zoom;

                self.ui.zoom *= zoom_multiplier;
                self.ui.zoom = self.ui.zoom.clamp(0.1, 5.0);
                self.ui.pan = pointer_vec - screen_center - graph_pos * self.ui.zoom;
            }
        }

        let to_screen = |p: egui::Pos2| -> egui::Pos2 { (screen_center + self.ui.pan + p.to_vec2() * self.ui.zoom).to_pos2() };
        let painter = ui.painter().with_clip_rect(area_to_fill);

        painter.rect_filled(area_to_fill, 0.0, self.ui.theme.painter_bg);

        let has_visible_nodes = nodes.iter().any(|n| n.visible);

        if !has_visible_nodes {
            let window_frame = egui::Frame::window(&ui.ctx().global_style())
                .fill(self.ui.theme.button_bg)
                .inner_margin(15.0)
                .corner_radius(8.0)
                .stroke(egui::Stroke::new(1.0_f32, self.ui.theme.edge_fg));

            egui::Window::new("empty_graph_info")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .frame(window_frame)
                .show(ui.ctx(), |ui| {
                    ui.style_mut().visuals.override_text_color = Some(self.ui.theme.text_fg);

                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Welcome to the LDM KG traversal Tool").size(18.0).strong());
                    });

                    ui.label(egui::RichText::new(
                        "This tool offers you the ability to graphically travers the content of the LDM KG."
                    ).size(15.0));

                    let features = [
                        "Start by selecting a start point and confirming it in the very top widget.",
                        "After loading a starting point, the Graph View will show that information graphically.",
                        "The Analytics View shows information about the loaded triples.",
                        "The Node Inspector View can show a tabular view of the information connected to a node.",
                        "The Export button exports the loaded data in different formats.",
                        "The Colour Mode button gives you the ability to switch between light and dark colour themes.",
                        "The graph in the Graph View can be reset with the Reset View button.",
                        "On the left is the Legend and further information on Graph Controls.",
                    ];

                    for feature in features {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("•").size(15.0).strong());
                            ui.label(egui::RichText::new(feature).size(15.0));
                        });
                    }

                });
        }

        let mut hovered_node = self.ui.dragged_node;

        if hovered_node.is_none() {
            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                // Reverse iteration ensures we select the top-most node if they overlap
                for (index, node) in nodes.iter().enumerate().rev() {
                    if !node.visible {
                        continue;
                    }
                    let screen_pos = to_screen(node.pos);
                    let radius = 15.0 * self.ui.zoom;
                    let rect = egui::Rect::from_center_size(screen_pos, egui::vec2(radius * 2.0, radius * 2.0));
                    if rect.contains(pointer_pos) {
                        hovered_node = Some(index);
                        break;
                    }
                }
            }
        }

        let mut connected_nodes = std::collections::HashSet::new();
        let mut connected_edges = std::collections::HashSet::new();

        if let Some(hovered_idx) = hovered_node {
            connected_nodes.insert(hovered_idx);
            for (e_idx, edge) in edges.iter().enumerate() {
                if !edge.visible {
                    continue;
                }
                if edge.source == hovered_idx {
                    connected_nodes.insert(edge.target);
                    connected_edges.insert(e_idx);
                } else if edge.target == hovered_idx {
                    connected_nodes.insert(edge.source);
                    connected_edges.insert(e_idx);
                }
            }
        }

        // ==============================================================
        // --- 4-STAGE RENDERING PASSES FOR CORRECT HIGHLIGHT Z-INDEX ---
        // ==============================================================

        // PASS 1: Draw ALL dimmed edges first (Bottom layer)
        for (edge_idx, edge) in edges.iter().enumerate() {
            if !edge.visible {
                continue;
            }
            let is_connected_edge = connected_edges.contains(&edge_idx);
            let is_dimmed = hovered_node.is_some() && !is_connected_edge;

            if is_dimmed {
                crate::draw_edge!(edge, is_connected_edge, true, self, painter, nodes, to_screen);
            }
        }

        let mut clicked_to_expand = None;
        let mut dragged_node_delta = None;
        let mut clicked_to_fetch = None;

        // Temporarily store highlighted nodes so we can defer drawing them to the end
        let mut nodes_to_draw_on_top = Vec::new();

        // track if any node is activle beeing dragged
        let mut any_dragged_node = false;

        // PASS 2: Handle node interactions and draw ONLY dimmed nodes
        for (index, node) in nodes.iter().enumerate() {
            if !node.visible {
                continue;
            }

            let screen_pos = to_screen(node.pos);
            let radius = 15.0 * self.ui.zoom;

            let response = ui.interact(
                egui::Rect::from_center_size(screen_pos, egui::vec2(radius * 2.0, radius * 2.0)),
                ui.id().with(&node.id),
                egui::Sense::click_and_drag(),
            );

            if response.dragged() {
                any_dragged_node = true;
                self.ui.dragged_node = Some(index);
            }

            let current_time = ui.input(|i| i.time);

            if response.dragged() {
                let delta = response.drag_delta() / self.ui.zoom;
                self.ui.selected_node = None;
                self.ui.pending_click_node = None;
                dragged_node_delta = Some((index, delta));
            }

            if response.double_clicked() {
                let is_fetchable = node.rdf_type.contains(crate::constants::TYPE_AUTHOR)
                    || node.rdf_type.contains(crate::constants::TYPE_DATASERVICE)
                    || node.rdf_type.contains(crate::constants::TYPE_DATASET)
                    || node.rdf_type.contains(crate::constants::TYPE_CONCEPT)
                    || node.rdf_type.contains(crate::constants::TYPE_ORGANIZATION);

                let needs_fetch = is_fetchable
                    && (!node.api_fetched || (node.rdf_type.contains(crate::constants::TYPE_ORGANIZATION) && node.has_more_to_fetch));

                if needs_fetch {
                    clicked_to_fetch = Some(index);
                } else {
                    clicked_to_expand = Some(index);
                }

                self.ui.selected_node = None;
                self.ui.pending_click_node = None;
            } else if response.clicked() {
                self.ui.pending_click_node = Some(index);
                self.ui.pending_click_time = current_time;
            }

            if response.secondary_clicked() {
                self.ui.selected_node = Some(index);
                self.ui.show_menu = true;
                self.ui.pending_click_node = None;
            }

            if self.ui.pending_click_node == Some(index) {
                if (current_time - self.ui.pending_click_time) > 0.25 {
                    if self.ui.selected_node == Some(index) && !self.ui.show_menu {
                        self.ui.selected_node = None;
                    } else {
                        self.ui.selected_node = Some(index);
                        self.ui.show_menu = false;
                    }
                    self.ui.pending_click_node = None;
                } else {
                    ui.ctx().request_repaint();
                }
            }

            let is_highlighted = hovered_node == Some(index) || connected_nodes.contains(&index);
            let is_dimmed = hovered_node.is_some() && !is_highlighted;

            if is_dimmed {
                // Draw dimmed node over dimmed edge
                crate::draw_node!(index, node, response, true, self, ctx, painter, edges, to_screen, draw_node_details);
            } else {
                // Save it for the top layer pass
                nodes_to_draw_on_top.push((index, response));
            }
        }

        if !any_dragged_node {
            self.ui.dragged_node = None;
        }

        // PASS 3: Draw highlighted (or normal) edges over dimmed nodes
        for (edge_idx, edge) in edges.iter().enumerate() {
            if !edge.visible {
                continue;
            }
            let is_connected_edge = connected_edges.contains(&edge_idx);
            let is_dimmed = hovered_node.is_some() && !is_connected_edge;

            if !is_dimmed {
                crate::draw_edge!(edge, is_connected_edge, false, self, painter, nodes, to_screen);
            }
        }

        // PASS 4: Finally, draw highlighted (or normal) nodes (Top layer)
        for (index, response) in nodes_to_draw_on_top {
            let node = &nodes[index];
            crate::draw_node!(
                index,
                node,
                response,
                false,
                self,
                ctx,
                painter,
                edges,
                to_screen,
                draw_node_details
            );
        }

        // node menu
        if let Some(menu_idx) = self.ui.selected_node {
            if self.ui.show_menu {
                let screen_pos = to_screen(nodes[menu_idx].pos);

                crate::node_menu::draw_radial_menu(
                    ui,
                    &ctx,
                    &painter,
                    menu_idx,
                    screen_pos,
                    self.ui.zoom,
                    &self.ui.theme,
                    nodes,
                    edges,
                    &self.config.api_url,
                    self.graph_data.clone(),
                    &mut self.ui.show_menu,
                    &mut self.ui.selected_node,
                    &mut clicked_to_expand,
                );
            }
        }

        // move child node in sync
        if let Some((parent_idx, delta)) = dragged_node_delta {
            nodes[parent_idx].pos += delta;
        }

        // expansion
        if let Some(parent_idx) = clicked_to_expand {
            let is_currently_expanded = nodes[parent_idx].expanded;

            if is_currently_expanded {
                // cascade collapse
                let mut stack = vec![parent_idx];

                while let Some(current_idx) = stack.pop() {
                    nodes[current_idx].expanded = false;

                    for edge_idx in 0..edges.len() {
                        let mut child_idx_opt = None;

                        if edges[edge_idx].source == current_idx && edges[edge_idx].visible {
                            child_idx_opt = Some(edges[edge_idx].target);
                        } else if edges[edge_idx].target == current_idx && edges[edge_idx].visible {
                            child_idx_opt = Some(edges[edge_idx].source);
                        }

                        if let Some(child_idx) = child_idx_opt {
                            edges[edge_idx].visible = false;

                            let has_other_active_parents =
                                edges.iter().any(|e| (e.target == child_idx || e.source == child_idx) && e.visible);

                            if !has_other_active_parents {
                                nodes[child_idx].visible = false;

                                if nodes[child_idx].expanded {
                                    stack.push(child_idx);
                                }
                            }
                        }
                    }
                }
            } else {
                nodes[parent_idx].expanded = true;
                let mut children_indices = Vec::new();

                for (edge_idx, edge) in edges.iter().enumerate() {
                    if edge.source == parent_idx {
                        children_indices.push((edge_idx, edge.target));
                    } else if edge.target == parent_idx {
                        children_indices.push((edge_idx, edge.source));
                    }
                }

                let total_children = children_indices.len();
                let mut angle: f32 = 0.0;
                let angle_step = std::f32::consts::TAU / (total_children.max(1) as f32);
                let spawn_radius = 240.0;

                for (edge_idx, target_idx) in children_indices {
                    let target_pos = nodes[parent_idx].pos + egui::vec2(angle.cos() * spawn_radius, angle.sin() * spawn_radius);
                    angle += angle_step;

                    if !nodes[target_idx].visible {
                        nodes[target_idx].pos = target_pos;
                    }
                    nodes[target_idx].visible = true;
                    edges[edge_idx].visible = true;
                }
            }
        }

        if let Some(fetch_idx) = clicked_to_fetch {
            let clicked_node_id = nodes[fetch_idx].id.clone();
            let clicked_node_label = nodes[fetch_idx].label.clone();
            let current_type = nodes[fetch_idx].rdf_type.clone();
            let api_url = self.config.api_url.clone();
            let state = self.graph_data.clone();

            if current_type.contains(crate::constants::TYPE_AUTHOR) {
                crate::api_client::fetch_author_information(
                    ctx.clone(),
                    state.clone(),
                    clicked_node_id.clone(),
                    clicked_node_id.clone(),
                    nodes[fetch_idx].fetch_offset,
                    49,
                    &api_url,
                );
            }
            if current_type.contains(crate::constants::TYPE_DATASERVICE) || current_type.contains(crate::constants::TYPE_DATASET) {
                crate::api_client::fetch_dataset_information(
                    ctx.clone(),
                    state.clone(),
                    clicked_node_id.clone(),
                    clicked_node_id.clone(),
                    nodes[fetch_idx].fetch_offset,
                    49,
                    &api_url,
                );
            }
            if current_type.contains(crate::constants::TYPE_CONCEPT) {
                crate::api_client::fetch_keyword_information(
                    ctx.clone(),
                    state.clone(),
                    clicked_node_id.clone(),
                    clicked_node_label,
                    nodes[fetch_idx].fetch_offset,
                    49,
                    &api_url,
                );
            }
            if current_type.contains(crate::constants::TYPE_ORGANIZATION) {
                crate::api_client::fetch_publisher_information(
                    ctx.clone(),
                    state.clone(),
                    clicked_node_id.clone(),
                    clicked_node_id.clone(),
                    nodes[fetch_idx].fetch_offset,
                    49,
                    &api_url,
                );
            }
        }
    }
}
