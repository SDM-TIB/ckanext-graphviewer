# Rust Wasm RDF Graph Visualizer for CKAN

This extension for CKAN enables the visualization of a TTL file.

The Wasm file creates a GET response to the TTL file of a called dataset and builds a graph from it.

The build JS and Wasm files are located in `ckanext/graphviewer/public/graph_viewer/`.

# How to Build

``` bash
$ trunk build --release --dist ckanext/graphviewer/public/graph_viewer/ --filehash false
$ rm ckanext/graphviewer/public/graph_viewer/index.html
```
