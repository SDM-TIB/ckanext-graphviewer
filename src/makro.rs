#[macro_export]
macro_rules! draw_edge {
    ($edge:expr, $is_connected_edge:expr, $is_dimmed:expr, $app:expr, $painter:expr, $nodes:expr, $to_screen:expr) => {
        let s = &$nodes[$edge.source];
        let t = &$nodes[$edge.target];

        let p1 = $to_screen(s.pos);
        let p2 = $to_screen(t.pos);
        let vector = p2 - p1;
        let length = vector.length();

        if length > 0.0 {
            let dir = vector / length;

            let stroke_width = if $is_connected_edge {
                3.0 * $app.ui.zoom
            } else {
                1.5 * $app.ui.zoom
            };

            let edge_color = if $is_dimmed {
                let c = $app.ui.theme.edge_fg;
                egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 100)
            } else {
                $app.ui.theme.edge_fg
            };

            let node_radius = 15.0 * $app.ui.zoom;

            let aa_overlap = 0.5 * $app.ui.zoom;
            let effective_node_radius = node_radius - aa_overlap;

            let arrow_len = 12.0 * $app.ui.zoom;
            let arrow_angle: f32 = 0.4;
            let line_angle = dir.y.atan2(dir.x);

            let arrowhead_base_offset = arrow_len * arrow_angle.cos();

            let p1_visual_start = if $edge.bidirectional {
                effective_node_radius + arrowhead_base_offset
            } else {
                effective_node_radius
            };
            let adjusted_p1 = p1 + (dir * p1_visual_start);

            let p2_visual_end = effective_node_radius + arrowhead_base_offset;
            let adjusted_p2 = p2 - (dir * p2_visual_end);

            if length > (p1_visual_start + p2_visual_end) {
                $painter.line_segment([adjusted_p1, adjusted_p2], egui::Stroke::new(stroke_width, edge_color));
            }

            // draw arrowhead
            let tip = p2 - (dir * effective_node_radius);
            let angle_left = line_angle - arrow_angle;
            let p_left = tip - egui::vec2(angle_left.cos(), angle_left.sin()) * arrow_len;
            let angle_right = line_angle + arrow_angle;
            let p_right = tip - egui::vec2(angle_right.cos(), angle_right.sin()) * arrow_len;

            $painter.add(egui::Shape::convex_polygon(
                vec![tip, p_left, p_right],
                edge_color,
                egui::Stroke::NONE,
            ));

            // draw arrowhead reverse
            if $edge.bidirectional {
                let tip_rev = p1 + (dir * effective_node_radius);
                let dir_rev = -dir;
                let line_angle_rev = dir_rev.y.atan2(dir_rev.x);

                let angle_left_rev = line_angle_rev - arrow_angle;
                let p_left_rev = tip_rev - egui::vec2(angle_left_rev.cos(), angle_left_rev.sin()) * arrow_len;
                let angle_right_rev = line_angle_rev + arrow_angle;
                let p_right_rev = tip_rev - egui::vec2(angle_right_rev.cos(), angle_right_rev.sin()) * arrow_len;

                $painter.add(egui::Shape::convex_polygon(
                    vec![tip_rev, p_left_rev, p_right_rev],
                    edge_color,
                    egui::Stroke::NONE,
                ));
            }

            // draw label
            let center_point = p1 + (dir * length * 0.5);
            let font_size = (10.0 * $app.ui.zoom).round();

            if font_size > 4.0 {
                let is_flipped = dir.x < 0.0;

                let display_text = if !is_flipped {
                    if let Some(rev) = &$edge.reverse_label {
                        format!("{} ->\n<- {}", $edge.label, rev)
                    } else if $edge.bidirectional {
                        format!("<- {} ->", $edge.label)
                    } else {
                        format!("{} ->", $edge.label)
                    }
                } else {
                    if let Some(rev) = &$edge.reverse_label {
                        format!("<- {}\n{} ->", $edge.label, rev)
                    } else if $edge.bidirectional {
                        format!("<- {} ->", $edge.label)
                    } else {
                        format!("<- {}", $edge.label)
                    }
                };

                let text_color = if $is_dimmed {
                    let c = $app.ui.theme.text_fg;
                    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 100)
                } else {
                    $app.ui.theme.text_fg
                };

                let galley = $painter.layout_no_wrap(display_text, egui::FontId::proportional(font_size), text_color);
                let size = galley.size();
                let padding = 3.0 * $app.ui.zoom;

                let snapped_center = egui::pos2(center_point.x.round(), center_point.y.round());
                let text_rect = egui::Rect::from_center_size(snapped_center, size);

                $painter.rect_filled(text_rect.expand(padding), 2.0 * $app.ui.zoom, $app.ui.theme.painter_bg);
                $painter.galley(text_rect.min, galley, text_color);
            }
        }
    };
}

#[macro_export]
macro_rules! draw_node {
    ($index:expr, $node:expr, $response:expr, $is_dimmed:expr, $app:expr, $ctx:expr, $painter:expr, $edges:expr, $to_screen:expr, $draw_node_details:expr) => {
        let screen_pos = $to_screen($node.pos);
        let radius = 15.0 * $app.ui.zoom;

        let is_pinned = $app.ui.selected_node == Some($index) && !$app.ui.show_menu;

        if is_pinned {
            let offset = egui::vec2(20.0 * $app.ui.zoom, 20.0 * $app.ui.zoom);

            egui::Window::new(format!("node_window_{}", $node.id))
                .fixed_pos(screen_pos + offset)
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::popup(&$ctx.global_style()))
                .show(&$ctx, |ui| {
                    ui.heading(&$node.label);
                    ui.separator();

                    $draw_node_details(ui, $node);

                    ui.add_space(5.0);
                    if ui.button("Close").clicked() {
                        $app.ui.selected_node = None;
                    }
                });
        }

        let node_theme = $app.ui.theme.get_node_colors(&$node.rdf_type);

        let color = if $response.hovered() {
            node_theme.hovered
        } else {
            node_theme.normal
        };

        let final_color = if $is_dimmed {
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 100)
        } else {
            color
        };

        $painter.circle_filled(screen_pos, radius, final_color);

        let font_size = 12.0 * $app.ui.zoom;
        if font_size > 4.0 {
            let display_text = if $node.label.len() > 50 {
                let pred_name = $edges
                    .iter()
                    .find(|e| e.target == $index)
                    .map(|e| e.label.clone())
                    .unwrap_or_else(|| "Dataset".to_string());
                let display_pred = {
                    let mut c = pred_name.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                };

                format!("{} (Click to show)", display_pred)
            } else {
                $node.label.clone()
            };

            let text_color = if $is_dimmed {
                let c = $app.ui.theme.text_fg;
                egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 100)
            } else {
                $app.ui.theme.text_fg
            };

            let galley = $painter.layout_no_wrap(display_text.to_string(), egui::FontId::proportional(font_size), text_color);

            let text_pos = screen_pos + egui::vec2(0.0, 20.0 * $app.ui.zoom);
            let text_rect = egui::Align2::CENTER_TOP.anchor_rect(egui::Rect::from_min_size(text_pos, galley.size()));

            $painter.rect_filled(text_rect.expand(2.0 * $app.ui.zoom), 2.0 * $app.ui.zoom, $app.ui.theme.painter_bg);
            $painter.galley(text_rect.min, galley, text_color);
        }
    };
}
