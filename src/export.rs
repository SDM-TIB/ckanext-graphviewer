use crate::graph_processor::{Edge, Node};
use crate::theme::Theme;
use eframe::egui::Color32;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

// helper to convert egui::Color32 to hex string
fn color_to_hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

// helper to safely truncate strings with an ellipsis if they are too long
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{}...", truncated)
    } else {
        text.to_string()
    }
}

// helper to escape XML control characters
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// Generates an SVG string containing the graph and an external information panel
pub fn generate_svg(nodes: &[Node], edges: &[Edge], theme: &Theme) -> String {
    // calculate the bounding box of the visible graph
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut visible_node_count = 0;

    for n in nodes {
        if !n.visible {
            continue;
        }
        visible_node_count += 1;
        if n.pos.x < min_x {
            min_x = n.pos.x;
        }
        if n.pos.x > max_x {
            max_x = n.pos.x;
        }
        if n.pos.y < min_y {
            min_y = n.pos.y;
        }
        if n.pos.y > max_y {
            max_y = n.pos.y;
        }
    }

    // fallback if the graph is empty
    if visible_node_count == 0 {
        min_x = 0.0;
        max_x = 800.0;
        min_y = 0.0;
        max_y = 600.0;
    }

    // define canvas dimensions
    // padding for node labels
    let padding_x = 180.0;
    let padding_y = 80.0;

    let graph_width = (max_x - min_x) + (padding_x * 2.0);
    let graph_height = (max_y - min_y) + (padding_y * 2.0);

    let panel_width = 300.0;
    let total_width = graph_width + panel_width;
    let total_height = graph_height.max(600.0);

    let bg_color = color_to_hex(theme.master_bg);
    let text_color = color_to_hex(theme.text_fg);
    let edge_color = color_to_hex(theme.edge_fg);

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" style=\"font-family: sans-serif;\">\n",
        total_width, total_height, total_width, total_height
    ));

    // arrowhead
    svg.push_str("  <defs>\n");
    // right pointing half arrow
    svg.push_str(
        "    <marker id=\"arrow-end\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"8\" markerHeight=\"8\" orient=\"auto\">\n",
    );
    svg.push_str(&format!(
        "      <path d=\"M 0 10 L 10 5 L 0 5 z\" fill=\"{}\" />\n",
        edge_color
    ));
    svg.push_str("    </marker>\n");

    // left pointing half arrow
    svg.push_str(
        "    <marker id=\"arrow-start\" viewBox=\"0 0 10 10\" refX=\"2\" refY=\"5\" markerWidth=\"8\" markerHeight=\"8\" orient=\"auto\">\n",
    );
    svg.push_str(&format!(
        "      <path d=\"M 10 0 L 0 5 L 10 5 z\" fill=\"{}\" />\n",
        edge_color
    ));
    svg.push_str("    </marker>\n");
    svg.push_str("  </defs>\n");

    // background
    svg.push_str(&format!(
        "  <rect width=\"100%\" height=\"100%\" fill=\"{}\" />\n",
        bg_color
    ));

    // graoh area
    svg.push_str("  <g id=\"graph_area\">\n");

    // draw edges
    for edge in edges {
        if !edge.visible {
            continue;
        }
        let source_pos = nodes[edge.source].pos;
        let target_pos = nodes[edge.target].pos;

        let x1 = source_pos.x - min_x + padding_x;
        let y1 = source_pos.y - min_y + padding_y;
        let x2 = target_pos.x - min_x + padding_x;
        let y2 = target_pos.y - min_y + padding_y;

        // arrowheads
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 0.1 {
            continue;
        }

        let ux = dx / dist;
        let uy = dy / dist;

        // pull the line back by 16px (14px node radius + 2px gap)
        let offset = 16.0;
        let start_x = if edge.bidirectional {
            x1 + ux * offset
        } else {
            x1
        };
        let start_y = if edge.bidirectional {
            y1 + uy * offset
        } else {
            y1
        };
        let end_x = x2 - ux * offset;
        let end_y = y2 - uy * offset;

        let marker_start = if edge.bidirectional {
            " marker-start=\"url(#arrow-start)\""
        } else {
            ""
        };
        let marker_end = " marker-end=\"url(#arrow-end)\"";

        // wrap the line in a transparency group to prevent overlap seams with the markers
        svg.push_str("    <g opacity=\"0.6\">\n");
        svg.push_str(&format!(
            "      <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"1.5\"{}{} />\n",
            start_x, start_y, end_x, end_y, edge_color, marker_start, marker_end
        ));
        svg.push_str("    </g>\n");

        // edge label
        let mid_x = (x1 + x2) / 2.0;
        let mid_y = (y1 + y2) / 2.0;

        let mut angle_deg = dy.atan2(dx).to_degrees();
        let mut is_flipped = false;

        // flip the table
        if angle_deg > 90.0 || angle_deg < -90.0 {
            angle_deg += 180.0;
            is_flipped = true;
        }

        let (fwd_y, rev_y) = if is_flipped {
            (-6.0, 14.0)
        } else {
            (14.0, -6.0)
        };

        // helper closure to stack multiple labels vertically
        let draw_label = |label_str: &str,
                          base_y: f32,
                          svg_out: &mut String| {
            let labels: Vec<&str> = label_str.split(", ").collect();
            let is_top = base_y < 0.0;
            let line_height = 10.0;

            let start_y = if is_top {
                base_y - ((labels.len() - 1) as f32 * line_height)
            } else {
                base_y
            };

            svg_out.push_str(&format!(
                "    <text transform=\"translate({:.1}, {:.1}) rotate({:.1})\" fill=\"{}\" font-size=\"10\" text-anchor=\"middle\" stroke=\"{}\" stroke-width=\"3\" paint-order=\"stroke\" opacity=\"0.9\">\n",
                mid_x, mid_y, angle_deg, text_color, bg_color
            ));

            for (i, lbl) in labels.iter().enumerate() {
                if i == 0 {
                    svg_out.push_str(&format!(
                        "      <tspan x=\"0\" y=\"{:.1}\">{}</tspan>\n",
                        start_y, lbl
                    ));
                } else {
                    svg_out.push_str(&format!(
                        "      <tspan x=\"0\" dy=\"{:.1}\">{}</tspan>\n",
                        line_height, lbl
                    ));
                }
            }
            svg_out.push_str("    </text>\n");
        };

        let escaped_label = escape_xml(&edge.label);
        draw_label(&escaped_label, fwd_y, &mut svg);

        if let Some(rev_label) = &edge.reverse_label {
            let escaped_rev_label = escape_xml(rev_label);
            draw_label(&escaped_rev_label, rev_y, &mut svg);
        }
    }

    // draw nodes
    for node in nodes {
        if !node.visible {
            continue;
        }

        // use padding_x and padding_y
        let x = node.pos.x - min_x + padding_x;
        let y = node.pos.y - min_y + padding_y;

        let node_color = theme.get_node_colors(&node.rdf_type).normal;
        let fill_hex = color_to_hex(node_color);
        let radius = 14.0;

        svg.push_str(&format!(
            "    <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\" />\n",
            x, y, radius, fill_hex, bg_color
        ));

        // node Label
        let display_label = escape_xml(&truncate_text(&node.label, 30));

        svg.push_str(&format!(
            "    <text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"12\" text-anchor=\"middle\">{}</text>\n",
            x,
            y + radius + 14.0,
            text_color,
            display_label
        ));
    }
    svg.push_str("  </g>\n");

    // info panel
    svg.push_str(&format!(
        "  <g id=\"info_panel\" transform=\"translate({}, 0)\">\n",
        graph_width
    ));

    // panel background
    let panel_bg = color_to_hex(theme.button_bg);
    svg.push_str(&format!(
        "    <rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\" />\n",
        panel_width, total_height, panel_bg
    ));

    // legend header
    svg.push_str(&format!(
        "    <text x=\"20\" y=\"155\" fill=\"{}\" font-size=\"18\" font-weight=\"bold\">Legend</text>\n",
        text_color
    ));

    // render Legend entries
    let mut current_y = 190.0;
    for (rdf_type, colors) in &theme.node_map {
        let clean_name = escape_xml(
            rdf_type
                .split('/')
                .last()
                .unwrap_or(rdf_type)
                .split('#')
                .last()
                .unwrap_or(rdf_type),
        );

        svg.push_str(&format!(
            "    <circle cx=\"30\" cy=\"{:.1}\" r=\"8\" fill=\"{}\" />\n",
            current_y - 4.0,
            color_to_hex(colors.normal)
        ));
        svg.push_str(&format!(
            "    <text x=\"50\" y=\"{:.1}\" fill=\"{}\" font-size=\"14\">{}</text>\n",
            current_y, text_color, clean_name
        ));
        current_y += 30.0;
    }

    svg.push_str("  </g>\n");
    svg.push_str("</svg>");

    svg
}

// generate a n3 string
pub fn generate_n3(triples: &[crate::parser::RawTriple]) -> String {
    let mut n3 = String::new();

    for t in triples {
        n3.push_str(&format!("{} {} {} .\n", t.subject, t.predicate, t.object));
    }

    n3
}

// generate a json string
pub fn generate_json(nodes: &[Node], edges: &[Edge]) -> String {
    let export_obj = serde_json::json!({
        "nodes": nodes.iter().filter(|n| n.visible).map(|n| {
            serde_json::json!({
                "id": n.id,
                "label": n.label,
                "rdf_type": n.rdf_type,
                "x": n.pos.x,
                "y": n.pos.y,
                "properties": n.properties.iter().map(|(k, v)| {
                    serde_json::json!({"predicate": k, "object": v})
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>(),
        "edges": edges.iter().filter(|e| e.visible).map(|e| {
            serde_json::json!({
                "source": nodes[e.source].id,
                "target": nodes[e.target].id,
                "label": e.label,
                "reverse_label": e.reverse_label,
                "bidirectional": e.bidirectional
            })
        }).collect::<Vec<_>>()
    });

    serde_json::to_string_pretty(&export_obj)
        .unwrap_or_else(|_| "{}".to_string())
}

// save file trigger
#[cfg(target_arch = "wasm32")]
pub fn save_file(filename: &str, content: &str, mime_type: &str) {
    use wasm_bindgen::JsCast;
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let array = js_sys::Array::new();
            array.push(&wasm_bindgen::JsValue::from_str(content));
            let options = web_sys::BlobPropertyBag::new();
            options.set_type(mime_type);
            if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(
                &array, &options,
            ) {
                if let Ok(url) =
                    web_sys::Url::create_object_url_with_blob(&blob)
                {
                    if let Ok(a) = document.create_element("a") {
                        let _ = a.set_attribute("href", &url);
                        let _ = a.set_attribute("download", filename);
                        if let Ok(html_a) = a.dyn_into::<web_sys::HtmlElement>()
                        {
                            html_a.click();
                        }
                    }
                    let _ = web_sys::Url::revoke_object_url(&url);
                }
            }
        }
    }
}

// jeet it to the project dir for standalone version
#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(filename: &str, content: &str, _mime_type: &str) {
    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let full_path = cwd.join(filename);

    if let Err(e) = std::fs::write(&full_path, content) {
        log::error!("Failed to save to {:?}: {}", full_path, e);
    } else {
        log::info!("Successfully exported file to: {:?}", full_path);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save_png_from_svg_web(svg_data: &str, filename: &str) {
    use wasm_bindgen::JsValue;

    let js_code = r#"
        const svgBlob = new Blob([svgString], {type: "image/svg+xml;charset=utf-8"});
        const url = URL.createObjectURL(svgBlob);

        const img = new Image();
        img.onload = () => {
            const canvas = document.createElement("canvas");

            const scale = 2; // 2x high-resolution multiplier
            canvas.width = img.width * scale;
            canvas.height = img.height * scale;

            const ctx = canvas.getContext("2d");
            ctx.scale(scale, scale);
            ctx.drawImage(img, 0, 0);

            URL.revokeObjectURL(url);

            const a = document.createElement("a");
            a.download = filename;
            a.href = canvas.toDataURL("image/png");

            // Firefox requires the link to be in the DOM to click it
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
        };
        img.src = url;
    "#;

    let func = js_sys::Function::new_with_args("svgString, filename", js_code);
    let _ = func.call2(
        &JsValue::NULL,
        &JsValue::from_str(svg_data),
        &JsValue::from_str(filename),
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_png_from_svg(svg_data: &str, filename: &str) {
    // resvg re-exports usvg and tiny_skia to prevent version conflicts
    use resvg::tiny_skia::{Pixmap, Transform};
    use resvg::usvg::{Options, Tree, fontdb};

    // load system fonts
    let mut font_db = fontdb::Database::new();
    font_db.load_system_fonts();

    let fallback_family = font_db
        .faces()
        .next()
        .and_then(|face| face.families.first())
        .map(|(name, _)| name.clone());

    if let Some(family_name) = fallback_family {
        font_db.set_sans_serif_family(family_name.as_str());
    } else {
        log::warn!(
            "No system fonts were found! Text will not render. Please install a font package."
        );
    }

    let mut opt = Options::default();

    // attach font to options
    opt.fontdb = std::sync::Arc::new(font_db);

    // parse the SVG string into a render tree and save it
    match Tree::from_str(svg_data, &opt) {
        Ok(tree) => {
            let size = tree.size().to_int_size();
            if let Some(mut pixmap) = Pixmap::new(size.width(), size.height()) {
                resvg::render(
                    &tree,
                    Transform::default(),
                    &mut pixmap.as_mut(),
                );

                let cwd = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                let full_path = cwd.join(filename);

                if let Err(e) = pixmap.save_png(&full_path) {
                    log::error!("Failed to save PNG from SVG: {}", e);
                } else {
                    log::info!(
                        "Successfully rendered and exported PNG to: {:?}",
                        full_path
                    );
                }
            } else {
                log::error!("Failed to allocate pixel buffer for PNG.");
            }
        }
        Err(e) => {
            log::error!("Failed to parse SVG for PNG rendering: {}", e);
        }
    }
}
