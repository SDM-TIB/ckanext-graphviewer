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
        ui.add_space(1.0);

        // render graph controls bar and buttons
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Graph Controls:").color(self.ui.theme.text_fg));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let reset_view_button =
                    egui::Button::new(egui::RichText::new("Reset View").color(self.ui.theme.text_fg)).fill(self.ui.theme.button_bg);

                // restore the view configuration of the inital snapshot
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

        // draw the infobox of a node
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

        // ui frame for legend and controls window
        let combined_outline = egui::Frame::window(&ui.ctx().global_style())
            .fill(self.ui.theme.button_bg)
            .inner_margin(3.0)
            .corner_radius(5.0)
            .stroke(egui::Stroke::new(2.0_f32, self.ui.theme.master_bg));

        egui::Window::new("legend_and_controls_window")
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 101.0))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .frame(combined_outline)
            .show(ui.ctx(), |ui| {
                ui.style_mut().visuals.override_text_color = Some(self.ui.theme.text_fg);

                let inner_frame = egui::Frame::NONE
                    .inner_margin(5.0)
                    .corner_radius(5.0)
                    .stroke(egui::Stroke::new(1.0_f32, self.ui.theme.edge_fg))
                    .fill(self.ui.theme.master_bg);

                // legend
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

                // controles
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

        // initialize viewport space and canvas constraints
        let background_rect = ui.available_rect_before_wrap();
        let area_to_fill = ui.available_rect_before_wrap();

        self.ui.canvas_rect = Some(area_to_fill);
        let screen_center = area_to_fill.center().to_vec2();

        // background panning
        let background_response = ui.interact(background_rect, ui.id().with("background"), egui::Sense::click_and_drag());
        if background_response.dragged() {
            self.ui.pan += background_response.drag_delta();
        }
        if background_response.clicked() {
            self.ui.selected_node = None;
        }

        // zoom
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

        // map internal graph positions to pixel space
        let to_screen = |p: egui::Pos2| -> egui::Pos2 { (screen_center + self.ui.pan + p.to_vec2() * self.ui.zoom).to_pos2() };
        let painter = ui.painter().with_clip_rect(area_to_fill);

        painter.rect_filled(area_to_fill, 0.0, self.ui.theme.painter_bg);

        // intro window
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
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 180.0))
                .frame(window_frame)
                .show(ui.ctx(), |ui| {
                    ui.set_max_width(700.0);

                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Welcome to the LDM KG traversal Tool")
                                   .size(18.0)
                                   .color(self.ui.theme.text_fg)
                                   .strong());
                    });

                    ui.label(egui::RichText::new(
                        "This tool offers you the ability to graphically travers the content of the LDM KG."
                    ).size(15.0).color(self.ui.theme.text_fg));

                    let features = [
                        vec![("Start by selecting a start point type (e.g., Author Name, Dataset Title, or Paper Title) from the dropdown menu in the top search bar.", false)],
                        vec![("Enter your search term and click ", false), ("Confirm ", true), ("to query the knowledge graph and build the initial view.", false)],
                        vec![("After loading a starting point, the ", false), ("Graph View ", true), ("will show that information graphically.", false)],
                        vec![("The ", false), ("Analytics View ", true), ("shows information about the loaded triples.", false)],
                        vec![("The ", false), ("Node Inspector View ", true), ("can show a tabular view of the information connected to a node.", false)],
                        vec![("The ", false), ("Export ", true), ("button exports the loaded data in different formats.", false)],
                        vec![("The ", false), ("Colour Mode ", true), ("button gives you the ability to switch between light and dark colour themes.", false)],
                        vec![("The ", false), ("Reset View ", true), ("button loads the initial view of the graph that was created after confirming a search.", false)],
                        vec![("On the left is the ", false), ("Legend ", true), ("and further information on Graph ", false), ("Controls.", true)],
                    ];

                    for feature_segments in features {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label(egui::RichText::new("• ").size(15.0).color(self.ui.theme.text_fg));

                            for (text, is_bold) in feature_segments {
                                let mut rich_text = egui::RichText::new(text).size(15.0);
                                if is_bold {
                                    rich_text = rich_text.color(self.ui.theme.menu_expand_bg);
                                } else {
                                    rich_text = rich_text.color(self.ui.theme.text_fg);
                                }
                                ui.label(rich_text);
                            }
                        });
                    }
                });
        }

        // hover logic
        let mut hovered_node = self.ui.dragged_node;
        let mut hovered_edge = None;

        if hovered_node.is_none() {
            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                // determine if a node is hovered. iterating in reverse prioritizes top-most overlapping nodes
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

                // if no node is currently hovered, calculate mouse distance to line segments to detect edge hovering
                if hovered_node.is_none() {
                    let hit_distance = 6.0 * self.ui.zoom;

                    for (index, edge) in edges.iter().enumerate().rev() {
                        if !edge.visible {
                            continue;
                        }

                        let p1 = to_screen(nodes[edge.source].pos);
                        let p2 = to_screen(nodes[edge.target].pos);

                        let line_vec = p2 - p1;
                        let line_len_sq = line_vec.length_sq();
                        let pt_vec = pointer_pos - p1;

                        // calculate the shortest distance from the mouse pointer to the edge's line segment
                        let distance = if line_len_sq < 1.0 {
                            pt_vec.length()
                        } else {
                            // find the projection point on the line using vector dot products, clamped between 0.0 and 1.0
                            let t = (pt_vec.x * line_vec.x + pt_vec.y * line_vec.y) / line_len_sq;
                            let t = t.clamp(0.0, 1.0);
                            let proj = p1 + line_vec * t;
                            (pointer_pos - proj).length()
                        };

                        if distance <= hit_distance {
                            hovered_edge = Some(index);
                            break;
                        }
                    }
                }
            }
        }

        // highlighted edges and nodes
        let mut connected_nodes = std::collections::HashSet::new();
        let mut connected_edges = std::collections::HashSet::new();
        let is_hovering_something = hovered_node.is_some() || hovered_edge.is_some();

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
        } else if let Some(hovered_idx) = hovered_edge {
            connected_edges.insert(hovered_idx);
            connected_nodes.insert(edges[hovered_idx].source);
            connected_nodes.insert(edges[hovered_idx].target);
        }

        // 4-stage rendering passes for correct z-index sorting
        // elements need to be rendered back-to-front so that highlighted
        // items always appear on top of dimmed out background items

        // pass 1: draw all dimmed (un-hovered) edges first at the lowest z-index
        for (edge_idx, edge) in edges.iter().enumerate() {
            if !edge.visible {
                continue;
            }
            let is_connected_edge = connected_edges.contains(&edge_idx);
            let is_dimmed = is_hovering_something && !is_connected_edge;

            if is_dimmed {
                crate::draw_edge!(edge, is_connected_edge, true, self, painter, nodes, to_screen);
            }
        }

        let mut clicked_to_expand = None;
        let mut dragged_node_delta = None;
        let mut clicked_to_fetch = None;
        let mut any_dragged_node = false;

        // temporarily store highlighted nodes to defer drawing them to the very end (pass 4)
        let mut nodes_to_draw_on_top = Vec::new();

        // pass 2: handle node interactions (clicks, drags) and draw only dimmed nodes
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
                // timer implementation to differentiate a single click (open infobox) from a double click
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
            let is_dimmed = is_hovering_something && !is_highlighted;

            if is_dimmed {
                // dimmed nodes are drawn immediately (above dimmed edges but below active elements)
                crate::draw_node!(index, node, response, true, self, ctx, painter, edges, to_screen, draw_node_details);
            } else {
                nodes_to_draw_on_top.push((index, response));
            }
        }

        if !any_dragged_node {
            self.ui.dragged_node = None;
        }

        // pass 3: draw highlighted edges. because this runs after pass 2, they overlay dimmed nodes
        for (edge_idx, edge) in edges.iter().enumerate() {
            if !edge.visible {
                continue;
            }
            let is_connected_edge = connected_edges.contains(&edge_idx);
            let is_dimmed = is_hovering_something && !is_connected_edge;

            if !is_dimmed {
                crate::draw_edge!(edge, is_connected_edge, false, self, painter, nodes, to_screen);
            }
        }

        // pass 4: draw highlighted nodes. they are guaranteed to be at the very top of the z-index stack
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

        // draw radial context menu for right-clicked nodes
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

        // update the internal coordinate mapping for nodes that were dragged this frame.
        if let Some((parent_idx, delta)) = dragged_node_delta {
            nodes[parent_idx].pos += delta;
        }

        // node expansion and cascade collapse logic
        if let Some(parent_idx) = clicked_to_expand {
            let is_currently_expanded = nodes[parent_idx].expanded;

            if is_currently_expanded {
                // traverse down the tree to hide children, stopping if a child is connected to another active parent
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
                             let has_other_active_parents = edges.iter().any(|e| {
                                (e.target == child_idx || e.source == child_idx)
                                    && e.visible
                                    && (e.source != current_idx && e.target != current_idx)
                            });

                            if !has_other_active_parents {
                                edges[edge_idx].visible = false;
                                nodes[child_idx].visible = false;

                                 if nodes[child_idx].expanded {
                                    stack.push(child_idx);
                                }
                            }
                        }
                    }
                }
            } else {
                // expanding: make all connected edges and children visible, arranging newly shown nodes in a circle around the parent
                nodes[parent_idx].expanded = true;
                let mut hidden_children = Vec::new();
                let mut visible_children_edges = Vec::new();

                for (edge_idx, edge) in edges.iter().enumerate() {
                    if edge.source == parent_idx {
                        if !nodes[edge.target].visible {
                            hidden_children.push((edge_idx, edge.target));
                        } else {
                            visible_children_edges.push(edge_idx);
                        }
                    } else if edge.target == parent_idx {
                        if !nodes[edge.source].visible {
                            hidden_children.push((edge_idx, edge.source));
                        } else {
                            visible_children_edges.push(edge_idx);
                        }
                    }
                }

                for edge_idx in visible_children_edges {
                    edges[edge_idx].visible = true;
                }

                let total_new_children = hidden_children.len();
                let mut angle: f32 = 0.0;
                let angle_step = std::f32::consts::TAU / (total_new_children.max(1) as f32);
                let spawn_radius = 240.0;

                for (edge_idx, target_idx) in hidden_children {
                    let target_pos = nodes[parent_idx].pos + egui::vec2(angle.cos() * spawn_radius, angle.sin() * spawn_radius);
                    angle += angle_step;

                    nodes[target_idx].pos = target_pos;
                    nodes[target_idx].visible = true;
                    edges[edge_idx].visible = true;
                }
            }
        }

        // fetch kg information
        if let Some(fetch_idx) = clicked_to_fetch {
            let clicked_node_id = nodes[fetch_idx].id.clone();
            let clicked_node_label = nodes[fetch_idx].label.clone();
            let current_type = nodes[fetch_idx].rdf_type.clone();
            let api_url = self.config.api_url.clone();
            let state = self.graph_data.clone();

            // fetch author
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

            // fetch dataset or dataservice
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

            // fetch keyword
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

            // fetch publisher
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
