# Rust Wasm RDF Graph Visualizer for Leibniz Data Manager

This extension for LDM enables the visualization and traversal of a knowledge Graph. This extension dynamiclly querys the knowledge graph as new nodes are expanded.

## Building the Project

The Rust toolchain needs to be present to build the project.
``` bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The WASM binary are build with trunk, install it with
``` bash
$ cargo install trunk --locked
```

To compile the Project as a standalone binary run
``` bash
$ cargo build --release
```

To compile the project as a web app run
``` bash
$ trunk build --release --dist ckanext/graphviewer/public/graph_viewer/ --filehash false
$ rm ckanext/graphviewer/public/graph_viewer/index.html
```

## Run the standalone binary
``` bash
$ cargo run --release
```

## Installing the plugin in a LDM instance

### presesnt env vars
CKANEXT__GRAPHVIEWER__APIIP__SUBPATH=/kg_exploration  
CKANEXT__KG_EXPLORATION__ENDPOINT=http://ldm_kg:8890/sparql

### dependencys

for the following projects a starting point configuration can be found at [LDM_Docker](https://github.com/SDM-TIB/LDM_Docker/blob/main/docker-compose.yml)

#### ckanext-kgcreation
The plugin creates a html container that offers the download of different file types as well as a graph link to open the graphviewer html container. project url: [ckanext-kgcreation](https://github.com/SDM-TIB/ckanext-kgcreation)

#### kg exploration API
The used python file can be found here [api.py](https://github.com/SDM-TIB/FAIR_RDM_APIs/knowledge_graph_managment/knowledge_graph_exploration/api/api.py). The api is also available via docker image ghcr.io/sdm-tib/fair_rdm_kg_exploration

#### virtuoso
This Docker container is the endpoint used by the kg creation API
