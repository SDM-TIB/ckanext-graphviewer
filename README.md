# Rust Wasm RDF Graph Visualizer for CKAN

This extension for CKAN enables the visualization and traversal of a knowledge Graph.

The extension dynamicly querys the knowledge graph as new nodes a expanded.

The build JS and Wasm files are located in `ckanext/graphviewer/public/graph_viewer/`.

## Prerequisites

The Rust toolchain needs to be present

``` bash
$ cargo install trunk --locked
```

## Run natively

``` bash
$ cargo run --release
```

## Build JS and WASM for Web

``` bash
$ trunk build --release --dist ckanext/graphviewer/public/graph_viewer/ --filehash false
$ rm ckanext/graphviewer/public/graph_viewer/index.html
```
## needed Enviroment Variables

CKANEXT__GRAPHVIEWER__APIIP__SUBPATH=/kg_exploration
