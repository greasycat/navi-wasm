# Refactors

Current file-size inventory for `navi-wasm`.

## Rust files over 800 lines outside `src/network/`

- none

## Repo files over 800 lines

Excludes generated output and lockfiles:

- skipped: `target/`, `pkg/`, `node_modules/`, `dist/`, `.git/`
- skipped: `Cargo.lock`

- `crates/navi_plot_core/src/network/layout.rs`: 1891 lines
- `demo/main.js`: 1776 lines
- `demo/tree/tree.js`: 1606 lines
- `demo/level-ordered-tree/level-ordered-tree.js`: 1597 lines
- `demo/network/network.js`: 1305 lines
- `crates/navi_plot_core/src/network/render.rs`: 1046 lines

## Recently completed

- `crates/navi_plot_wasm/src/lib.rs`: split into `crates/navi_plot_wasm/src/wasm_impl/`
- `crates/navi_plot_core/src/tree.rs`: split into `crates/navi_plot_core/src/tree/`
- `crates/navi_plot_core/src/network/tests.rs`: split into `crates/navi_plot_core/src/network/tests/`

## Next obvious refactor targets

- `crates/navi_plot_core/src/network/layout.rs`
- `crates/navi_plot_core/src/network/render.rs`
