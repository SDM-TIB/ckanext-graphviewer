# LDM Knowledge Graph Traversal Tool

The **LDM Knowledge Graph Traversal Tool** operates as a high-performance, Rust-based visualizer and analytical dashboard. It is designed for exploring knowledge graphs within the Leibniz Data Manager (LDM) and CKAN ecosystems. By compiling to both native desktop binaries and WebAssembly (WASM), the tool delivers a deeply integrated, interactive experience directly within browser environments.

## Main Features

- **Interactive Graph Traversal:** Explore relationships dynamically using a force-directed, circular layout engine. Drag, pan, zoom, and double-click to fetch and expand connected nodes in real-time.
- **CKAN-Integrated Search:** A built-in autocomplete search bar connects directly to CKAN's `package_search` API, providing live suggestions for Authors, Dataset Titles, and DOIs.
- **Analytics & Metadata Dashboard:** Instantly view network health, ontology statistics (classes, instances, properties), and triple composition.
- **Node Inspector:** A searchable tabular view of all loaded entities, detailing their Subject and Object relationships.
- **Multi-Format Parsing & Export:** Natively parses N3 files and dynamic JSON API responses. Exports the active graph view to **SVG, PNG, JSON, and N3**.
- **Dynamic Configuration:** Automatically configures itself based on DOM attributes when embedded via WebAssembly, allowing seamless integration across different instances.

## Repository Structure

| Path | Purpose |
|---|---|
| `build.sh` | Shell script to compile the WebAssembly package via Trunk |
| `ckanext/graphviewer/` | Python package containing the CKAN extension logic |
| `ckanext/.../public/graph_viewer/` | Compiled WebAssembly assets (`rdf_wasm_graph.js`, `rdf_wasm_graph_bg.wasm`) |
| `ckanext/.../templates/` | HTML templates (`header.html`, `graph_viewer.html`) for CKAN integration |
| `src/main.rs` | Application entry point, state management, and configuration |
| `src/ui/` | Modular UI scenes including the graph, analytics, inspector, and search bar |
| `src/api_client.rs` | Backend integration, pagination, and graph expansion logic |
| `src/parser.rs` | N3 and Dynamic JSON parsing into internal `RawTriple` structs |
| `src/graph_processor.rs` | Topology construction, and layout algorithms |
| `src/export.rs` | Conversion pipelines for N3, JSON, SVG, and high-res PNG generation |

---

## Added Routes

The extension implements `IConfigurer` and `IBlueprint` to register the public assets, templates, and routing. It supports dynamic routing for both a global viewer and a dataset-specific viewer context:
- `/graph` -> Global viewer mode.
- `/<_type>/<_id>/graph` -> Dataset-specific mode (passes the `pkg_dict` to pre-load relevant dataset information).

## Building the Application

The tool is written in Rust and relies on the `egui` framework (via `eframe`). It can be built as a standalone desktop application or as a WebAssembly module.

### Prerequisites

- [Rust Toolchain](https://rustup.rs/) (1.70 or newer recommended)
- `trunk` (for WebAssembly builds)

Install the WASM target and Trunk:
```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

### Native Desktop Build (Development/Testing)
For local testing outside of a browser, run the native build:
```bash
cargo run --release
```

### WebAssembly Build (Production Integration)
To build the module for CKAN/LDM integration, use the provided `build.sh` script. This utilizes `trunk` to compile the WebAssembly package and outputs it directly into the CKAN extension's public directory without file hashing, removing the default `index.html` file since CKAN serves its own templates.

Execute the build script from your shell:
```bash
./build.sh
```

## Integration Guide: CKAN & LDM

For the extension to function correctly, its underlying microservice dependencies must be properly orchestrated.

### Virtuoso Triplestore

The following docker-compose snippet describs the deployment of a virtuoso container that serves the purpose of beeing the endpoint that handels sparql querys created be the kg_exploration API

``` yaml
ldm_kg:
  container_name: ldm_kg
  image: kemele/virtuoso:7-stable
  restart: unless-stopped
  ports:
    - "8890:8890"
  volumes:
    - kg_data:/data
    - ./rdf_metadata:/data/toLoad
  networks:
    - ldmnetwork
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.ldmkg.rule=Path(`/ldmkg/sparql`)"
    - "traefik.http.routers.ldmkg.priority=10"
    - "traefik.http.routers.ldmkg.middlewares=ldmkg-rewrite"
    - "traefik.http.middlewares.ldmkg-rewrite.replacepathregex.regex=^/ldmkg/sparql(.*)"
    - "traefik.http.middlewares.ldmkg-rewrite.replacepathregex.replacement=/sparql$$1"
    - "traefik.http.services.ldmkg.loadbalancer.server.port=8890"
    - "traefik.http.routers.ldmkg-redirect.rule=Path(`/ldmkg`)"
    - "traefik.http.routers.ldmkg-redirect.priority=10"
    - "traefik.http.routers.ldmkg-redirect.middlewares=ldmkg-redirect-to-sparql"
    - "traefik.http.middlewares.ldmkg-redirect-to-sparql.redirectregex.regex=^(.*)/ldmkg$$"
    - "traefik.http.middlewares.ldmkg-redirect-to-sparql.redirectregex.replacement=$${1}/ldmkg/sparql"
    - "traefik.http.middlewares.ldmkg-redirect-to-sparql.redirectregex.permanent=true"

```

### Exploration API

This service utilizes the fair_rdm_kg_exploration image to bridge the Virtuoso endpoint with the graph viewer plugin.

``` yaml
kg_exploration:
  container_name: kg_exploration
  restart: unless-stopped
  image: ghcr.io/sdm-tib/fair_rdm_kg_exploration:054f336
  ports:
    - "5742:5742"
  networks:
    - ldmnetwork
  env_file:
    - .env
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.kge.rule=PathPrefix(`${ROOT_PATH}${CKANEXT__GRAPHVIEWER__APIIP__SUBPATH}`)"
    - "traefik.http.routers.kge.priority=10"
    - "traefik.http.middlewares.kge-stripprefix.stripprefix.prefixes=${ROOT_PATH}${CKANEXT__GRAPHVIEWER__APIIP__SUBPATH}"
    - "traefik.http.routers.kge.middlewares=kge-stripprefix"
    - "traefik.http.services.kge.loadbalancer.server.port=5742"

```
### Plugin dependency

| Plugin | Purpose |
|---|---|
| TIBtheme | display of datasets |
| kgcreation | display of exports of a dataset |

---

### Treafik Edge Router

Traefik is deployed as a reverse proxy to resolve the routing between the container.

### Environment Configuration

To ensure seamless communication between the frontend plugin and the backend services, specific environment variables must be exposed by CKAN.

``` bash
CKANEXT__GRAPHVIEWER__APIIP__SUBPATH=/kg_exploration
CKANEXT__KG_EXPLORATION__ENDPOINT=http://ldm_kg:8890/sparql

```

### Graphviewer Installation

To install the plugin in a CKAN instance follow these steps

#### Clone the plugin

execute a git clone in the ckan container

``` bash
cd /lib/ckan/default/src/
git clone git@github.com:SDM-TIB/ckanext-graphviewer.git
```

#### install the plugin

``` bash
cd ckanext-graphviewer
ckan-pip install -e .
```

#### enable the plugin

ensure graph_viewer is added the plugins in ckan.ini via editing the ckan.ini or ckan-entrypoint.sh

``` bash
sed -i '/^\s*ckan\.plugins\s*=/ {/graph_viewer/! s/=/= graph_viewer /}' /etc/ckan/default/ckan.ini
```

## Contact & Support
Developed by Wolfgang Schröder &lt;wolfgang.schroeder@proton.me&gt;

