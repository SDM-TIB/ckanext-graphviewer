pub mod api_client;
pub mod constants;
pub mod export;
mod graph_processor;
mod node_menu;
mod parser;
mod theme;
pub mod ui;

use eframe::egui;
use log::info;
use serde::Deserialize;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use graph_processor::{Edge, Node};
use theme::Theme;

#[derive(Deserialize, Debug)]
pub struct CkanResponse {
    pub result: CkanResult,
}

#[derive(Deserialize, Debug)]
pub struct CkanResult {
    pub results: Vec<CkanDataset>,
}

#[derive(Deserialize, Debug)]
pub struct CkanDataset {
    pub author: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct FoundString {
    pub name: String,
}

pub enum FetchMessage {
    Success(Vec<FoundString>),
    Error(String),
}

#[derive(PartialEq)]
pub enum Scene {
    Graph,
    Analytics,
    NodeInspector,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum SearchType {
    AuthorName,
    AuthorOrcid,
    AuthorLdmId,
    PaperDoi,
    PaperTitle,
    DatasetDoi,
    DatasetTitle,
    DatasetLdmId,
}

#[derive(PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    TestingRed,
}

impl SearchType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchType::AuthorName => "Author Name",
            SearchType::AuthorOrcid => "Author ORCID",
            SearchType::AuthorLdmId => "Author LDM ID",
            SearchType::PaperDoi => "Paper DOI",
            SearchType::PaperTitle => "Paper Title",
            SearchType::DatasetDoi => "Dataset DOI",
            SearchType::DatasetTitle => "Dataset Title",
            SearchType::DatasetLdmId => "Dataset LDM ID",
        }
    }

    pub fn all() -> Vec<SearchType> {
        vec![
            SearchType::AuthorName,
            SearchType::AuthorOrcid,
            SearchType::AuthorLdmId,
            SearchType::PaperDoi,
            SearchType::PaperTitle,
            SearchType::DatasetDoi,
            SearchType::DatasetTitle,
            SearchType::DatasetLdmId,
        ]
    }
}

#[derive(Clone)]
pub struct GraphSnapshot {
    pub node_positions: std::collections::HashMap<String, egui::Pos2>,
    pub visible_nodes: std::collections::HashSet<String>,
    pub expanded_nodes: std::collections::HashSet<String>,
    pub visible_edges: std::collections::HashSet<(String, String)>,
}

impl GraphSnapshot {
    pub fn new(nodes: &[Node], edges: &[Edge]) -> Self {
        let mut node_positions = std::collections::HashMap::new();
        let mut visible_nodes = std::collections::HashSet::new();
        let mut expanded_nodes = std::collections::HashSet::new();
        let mut visible_edges = std::collections::HashSet::new();

        for n in nodes {
            node_positions.insert(n.id.clone(), n.pos);
            if n.visible {
                visible_nodes.insert(n.id.clone());
            }
            if n.expanded {
                expanded_nodes.insert(n.id.clone());
            }
        }
        for e in edges {
            if e.visible {
                visible_edges.insert((nodes[e.source].id.clone(), nodes[e.target].id.clone()));
            }
        }

        Self {
            node_positions,
            visible_nodes,
            expanded_nodes,
            visible_edges,
        }
    }
}

pub enum AppState {
    Loading,
    Error(String),
    Ready {
        nodes: Vec<Node>,
        edges: Vec<Edge>,
        raw_triples: Vec<parser::RawTriple>,
        init_snapshot: GraphSnapshot,
    },
}

pub struct AppConfig {
    pub api_url: String,
    pub ckan_url: String,
    pub is_global_viewer: bool,
    pub rows_per_page: usize,
}

pub struct SearchState {
    pub search_type: SearchType,
    pub search_input: String,
    pub highlighted_index: usize,
    pub is_fetching: Arc<Mutex<bool>>,
    pub search_failed: Arc<Mutex<bool>>,
    pub autocomplete_rx: Receiver<FetchMessage>,
    pub autocomplete_tx: Sender<FetchMessage>,
    pub found_strings: Vec<FoundString>,
    pub autocomplete_fetching: bool,
    pub current_offset: usize,
}

pub struct UIState {
    // Theme & Navigation
    pub theme: Theme,
    pub theme_mode: ThemeMode,
    pub current_scene: Scene,

    // Viewport / Camera
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub canvas_rect: Option<egui::Rect>,

    // Graph Interaction
    pub selected_node: Option<usize>,
    pub show_menu: bool,
    pub pending_click_node: Option<usize>,
    pub pending_click_time: f64,

    // Inspector Panel
    pub inspector_selected_node: Option<String>,
    pub inspector_search_text: String,
}

struct App {
    pub config: AppConfig,
    pub graph_data: Arc<Mutex<AppState>>,
    pub search: SearchState,
    pub ui: UIState,
}

#[cfg(target_arch = "wasm32")]
pub fn get_api_url() -> String {
    let mut api_protocol = String::from("");
    let mut api_ip = String::from("");
    let mut api_port = String::from("");

    if let Some(window) = web_sys::window() {
        let document = window.document();

        if let Some(canvas) = document.expect("Failed to read canvas id").get_element_by_id("the_canvas_id") {
            let Some(protocol) = canvas.get_attribute("api_protocol") else { todo!() };
            let Some(ip) = canvas.get_attribute("api_ip") else { todo!() };
            let Some(port) = canvas.get_attribute("api_port") else { todo!() };

            if !protocol.is_empty() && !ip.is_empty() && !port.is_empty(){
                api_protocol = protocol;
                api_ip = ip;
                api_port = port;
            }
        }

        return format!("{}//{}:{}", api_protocol, api_ip, api_port);
    }
    // fallback
    "http://127.0.0.1:5742".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_api_url() -> String {
    // assume local same ip api
    "http://127.0.0.1:5742".to_string()
}

#[cfg(target_arch = "wasm32")]
pub fn get_dynamic_ckan_url() -> String {
    if let Some(window) = web_sys::window() {
        let location = window.location();

        // Extract the protocol (e.g., "http:") and hostname (e.g., "192.168.1.50" or "example.com")
        if let (Ok(protocol), Ok(hostname), Ok(port)) = (location.protocol(), location.hostname(), location.port()) {
            // Construct the target URL pointing to your Python FastAPI port
            return format!("{}//{}:{}", protocol, hostname, port);
        }
    }
    // Fallback if browser APIs fail
    "http://127.0.0.1:5000".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_dynamic_ckan_url() -> String {
    // Native desktop apps run locally, so they point to the local API
    "http://127.0.0.1:5000".to_string()
}

#[cfg(target_arch = "wasm32")]
pub fn get_n3_url_from_dom() -> Option<String> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let location = window.location();
    let canvas = document.get_element_by_id("the_canvas_id")?;
    let n3_path = canvas.get_attribute("data-n3")?;
    let mut origin = String::from("");

    if n3_path.starts_with("http://") || n3_path.starts_with("https://") {
        return Some(n3_path);
    }

    if let (Ok(protocol), Ok(hostname), Ok(port)) = (location.protocol(), location.hostname(), location.port()) {
        // Construct the target URL pointing to your Python FastAPI port
        origin = format!("{}//{}:{}", protocol, hostname, port);
    }

    let clean_path = n3_path.trim_start_matches('/');

    Some(format!("{}/{}", origin, clean_path))
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.global_style_mut(|style| {
            style.interaction.tooltip_delay = 0.0;
        });

        let is_system_dark = cc.egui_ctx.global_style().visuals.dark_mode;

        if is_system_dark {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        let state = Arc::new(Mutex::new(AppState::Loading));

        let api_url = get_api_url();

        let ckan_url = get_dynamic_ckan_url();

        #[cfg(target_arch = "wasm32")]
        let n3_target_url = get_n3_url_from_dom();
        #[cfg(not(target_arch = "wasm32"))]
        let n3_target_url: Option<String> = None;

        let is_global_viewer = {
            #[cfg(target_arch = "wasm32")]
            {
                get_n3_url_from_dom().is_none()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                true
            }
        };

        // Fetch N3 File (If Applicable) or mark Ready directly
        if let Some(target_url) = n3_target_url {
            let state_guard_clone = state.clone();
            let ctx_guard_clone = cc.egui_ctx.clone();

            let request = ehttp::Request::get(&target_url);
            ehttp::fetch(request, move |response| {
                match response {
                    Ok(res) => {
                        if let Some(text) = res.text() {
                            let raw_triples = parser::parse_n3_file(&text);
                            let (nodes, edges) = graph_processor::build_ui_graph(raw_triples.clone());
                            let init_snapshot = GraphSnapshot::new(&nodes, &edges);

                            *state_guard_clone.lock().unwrap() = AppState::Ready {
                                nodes,
                                edges,
                                raw_triples,
                                init_snapshot,
                            };
                        } else {
                            *state_guard_clone.lock().unwrap() = AppState::Error("failed to read text from n3".into());
                        }
                    }
                    Err(err) => {
                        *state_guard_clone.lock().unwrap() = AppState::Error(format!("Network Error: {}", err));
                    }
                }
                ctx_guard_clone.request_repaint();
            });
        } else {
            // Global viewer without predefined n3 file, jump straight to Ready
            *state.lock().unwrap() = AppState::Ready {
                nodes: Vec::new(),
                edges: Vec::new(),
                raw_triples: Vec::new(),
                init_snapshot: GraphSnapshot::new(&[], &[]),
            };
        }

        let (autocomplete_tx, autocomplete_rx) = mpsc::channel();

        Self {
            config: AppConfig {
                api_url: api_url,
                ckan_url: ckan_url,
                is_global_viewer: is_global_viewer,
                rows_per_page: 100,
            },
            graph_data: state,
            search: SearchState {
                search_type: SearchType::AuthorName,
                search_input: String::new(),
                highlighted_index: 0,
                is_fetching: Arc::new(Mutex::new(false)),
                search_failed: Arc::new(Mutex::new(false)),
                autocomplete_rx,
                autocomplete_tx,
                found_strings: Vec::new(),
                autocomplete_fetching: false,
                current_offset: 0,
            },
            ui: UIState {
                theme: Theme::dark(),
                theme_mode: ThemeMode::Dark,
                current_scene: Scene::Graph,
                zoom: 1.0,
                pan: egui::vec2(0.0, 0.0),
                canvas_rect: None,
                selected_node: None,
                show_menu: false,
                pending_click_node: None,
                pending_click_time: 0.0,
                inspector_selected_node: None,
                inspector_search_text: String::new(),
            },
        }
    }

    pub fn trigger_autocomplete_fetch(&self) {
        // 1. Determine which Solr field we are querying based on the dropdown
        let (field, value) = match self.search.search_type {
            SearchType::AuthorName => ("author", urlencoding::encode(&self.search.search_input).into_owned()),
            SearchType::DatasetTitle => ("title", urlencoding::encode(&self.search.search_input).into_owned()),
            _ => return, // Ignore auto-complete for other search types
        };

        let query_string = format!(
            "?q={}:{}~&fl={}&rows={}&start={}",
            field, value, field, self.config.rows_per_page, self.search.current_offset
        );

        // Hitting the CKAN endpoint directly for suggestions
        let request = ehttp::Request::get(format!(
            "{}/api/3/action/package_search{}",
            self.config.ckan_url,
            query_string
        ));

        let tx = self.search.autocomplete_tx.clone();
        let search_type_clone = self.search.search_type.clone();

        ehttp::fetch(request, move |result| {
            if let Ok(message) = result {
                if let Ok(parsed_data) = serde_json::from_slice::<CkanResponse>(&message.bytes) {
                    let mut new_results = Vec::new();
                    for dataset in parsed_data.result.results {
                        // 2. Extract the correct field from the JSON response
                        if search_type_clone == SearchType::AuthorName {
                            if let Some(val) = dataset.author {
                                new_results.push(FoundString { name: val });
                            }
                        } else if search_type_clone == SearchType::DatasetTitle {
                            if let Some(val) = dataset.title {
                                new_results.push(FoundString { name: val });
                            }
                        }
                    }
                    let _ = tx.send(FetchMessage::Success(new_results));
                }
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, app_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = app_ui.ctx().clone();

        ctx.set_visuals(self.ui.theme.to_egui_visuals());

        let main_app_frame = egui::Frame::central_panel(&ctx.global_style())
            .fill(self.ui.theme.master_bg)
            .inner_margin(6.0);

        egui::CentralPanel::default().frame(main_app_frame).show_inside(app_ui, |ui| {
            let state_arc = self.graph_data.clone();
            let mut state_lock = state_arc.lock().unwrap();

            if let AppState::Ready { .. } = &mut *state_lock {
                self.render_search_bar(ui, &ctx);
            }
            match &mut *state_lock {
                AppState::Ready {
                    nodes,
                    edges,
                    raw_triples,
                    init_snapshot,
                } => {
                    ui.spacing_mut().interact_size.y = 19.0;

                    // scene select tabs
                    ui.add_space(1.0); // to ocd or not to ocd
                    ui.horizontal(|ui| {
                        let graph_bg = if self.ui.current_scene == crate::Scene::Graph {
                            self.ui.theme.menu_expand_bg
                        } else {
                            self.ui.theme.button_bg
                        };
                        if ui.add(egui::Button::new("Graph View").fill(graph_bg)).clicked() {
                            self.ui.current_scene = crate::Scene::Graph;
                        }

                        let analytics_bg = if self.ui.current_scene == crate::Scene::Analytics {
                            self.ui.theme.menu_expand_bg
                        } else {
                            self.ui.theme.button_bg
                        };
                        if ui.add(egui::Button::new("Analytics View").fill(analytics_bg)).clicked() {
                            self.ui.current_scene = crate::Scene::Analytics;
                        }

                        let inspector_bg = if self.ui.current_scene == crate::Scene::NodeInspector {
                            self.ui.theme.menu_expand_bg
                        } else {
                            self.ui.theme.button_bg
                        };
                        if ui.add(egui::Button::new("Node Inspector View").fill(inspector_bg)).clicked() {
                            self.ui.current_scene = crate::Scene::NodeInspector;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // theme button
                            let theme_string = match self.ui.theme_mode {
                                ThemeMode::Dark => "Dark Mode",
                                ThemeMode::Light => "Light Mode",
                                ThemeMode::TestingRed => "Testing Red",
                            };

                            let theme_button = egui::Button::new(theme_string);

                            if ui.add(theme_button).clicked() {
                                match self.ui.theme_mode {
                                    ThemeMode::Dark => {
                                        self.ui.theme_mode = ThemeMode::Light;
                                        self.ui.theme = Theme::light();
                                    }
                                    ThemeMode::Light => {
                                        self.ui.theme_mode = ThemeMode::Dark;
                                        self.ui.theme = Theme::dark();
                                    }
                                    ThemeMode::TestingRed => {
                                        self.ui.theme_mode = ThemeMode::TestingRed;
                                        self.ui.theme = Theme::testing_red();
                                    }
                                }
                            }

                            ui.menu_button("Export", |ui| {
                                // Generate the ISO-like timestamp string
                                let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M").to_string();

                                if ui.button("Export as SVG").clicked() {
                                    let filename = format!("LDM_graph_export_{}.svg", timestamp);
                                    let svg_data = crate::export::generate_svg(nodes, edges, &self.ui.theme);
                                    crate::export::save_file(&filename, &svg_data, "image/svg+xml");
                                    ui.close();
                                }

                                if ui.button("Export as PNG").clicked() {
                                    let filename = format!("LDM_graph_export_{}.png", timestamp);
                                    let svg_data = crate::export::generate_svg(nodes, edges, &self.ui.theme);

                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        crate::export::save_png_from_svg_web(&svg_data, &filename);
                                        log::info!("Triggered WASM SVG-to-PNG download");
                                    }

                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        crate::export::save_png_from_svg(&svg_data, &filename);
                                    }

                                    ui.close();
                                }

                                if ui.button("Export as N3").clicked() {
                                    let filename = format!("LDM_graph_export_{}.n3", timestamp);
                                    let n3_data = crate::export::generate_n3(raw_triples);
                                    crate::export::save_file(&filename, &n3_data, "text/n3");
                                    ui.close();
                                }

                                if ui.button("Export as JSON").clicked() {
                                    let filename = format!("LDM_graph_export_{}.json", timestamp);
                                    let json_data = crate::export::generate_json(nodes, edges);
                                    crate::export::save_file(&filename, &json_data, "application/json");
                                    ui.close();
                                }
                            });
                        });
                    });
                    ui.separator();

                    match self.ui.current_scene {
                        crate::Scene::Graph => {
                            self.render_graph_scene(ui, &ctx, nodes, edges, init_snapshot);
                        }
                        crate::Scene::Analytics => {
                            self.render_analytics_scene(ui, nodes, edges, raw_triples);
                        }
                        crate::Scene::NodeInspector => {
                            self.render_inspector_scene(ui, nodes, edges);
                        }
                    }
                }
                AppState::Error(err_msg) => {
                    ui.heading("Something went wrong:");
                    ui.label(egui::RichText::new(err_msg.as_str()).color(self.ui.theme.error_fg).strong());
                }
                AppState::Loading => {
                    ui.heading("Loading Workspace and Fetching Dictionaries...");
                    ui.add(egui::Spinner::new());
                }
            }
        });
    }
}

// native entrypoint
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native("Standalone Test App", native_options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

// wasm entrypoint
#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let _ = js_sys::eval(
        r#"
        const ogGetContext = HTMLCanvasElement.prototype.getContext;
        HTMLCanvasElement.prototype.getContext = function(type, attrs) {
            if (type === 'webgl' || type === 'webgl2') {
                attrs = Object.assign({}, attrs || {}, { preserveDrawingBuffer: true });
            }
            return ogGetContext.call(this, type, attrs);
        };
    "#,
    );

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        use wasm_bindgen::JsCast;

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find 'the_canvas_id' in DOM")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element was not a HtmlCanvasElement");

        eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
            .await
            .expect("failed to start eframe");
    });
}
