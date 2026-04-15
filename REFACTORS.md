# Refactors

Current file-size inventory for `navi-wasm`.

## Rust files over 800 lines outside `src/network/`

- `crates/navi_plot_wasm/src/lib.rs`: 2606 lines
- `crates/navi_plot_core/src/tree.rs`: 1651 lines

## Repo files over 800 lines

Excludes generated output and lockfiles:

- skipped: `target/`, `pkg/`, `node_modules/`, `dist/`, `.git/`
- skipped: `Cargo.lock`

- `crates/navi_plot_wasm/src/lib.rs`: 2606 lines
- `crates/navi_plot_core/src/network/tests.rs`: 2020 lines
- `crates/navi_plot_core/src/network/layout.rs`: 1891 lines
- `demo/main.js`: 1776 lines
- `crates/navi_plot_core/src/tree.rs`: 1651 lines
- `demo/tree/tree.js`: 1606 lines
- `demo/level-ordered-tree/level-ordered-tree.js`: 1597 lines
- `demo/network/network.js`: 1305 lines
- `crates/navi_plot_core/src/network/render.rs`: 1046 lines

## Next obvious refactor targets

- `crates/navi_plot_wasm/src/lib.rs`
- `crates/navi_plot_core/src/tree.rs`
