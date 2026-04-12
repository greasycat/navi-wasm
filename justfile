# navi-wasm justfile — run `just --list` to see all recipes.

# Build the WASM package and link TypeScript spec types
build:
    wasm-pack build crates/navi_plot_wasm --target web --out-dir ../../pkg
    echo '/// <reference types="../types/navi_plot_specs.d.ts" />' >> pkg/navi_plot_wasm.d.ts

# Run all navi_plot_core tests
test:
    cargo test -p navi_plot_core

# Lint the core crate
check:
    cargo clippy -p navi_plot_core -- -D warnings

# Format all crates
fmt:
    cargo fmt --all

# Start the demo HTTP server (requires a prior `just build`)
serve:
    cargo run -p server

# Build the WASM package then start the demo server
dev: build serve
