# Refactors

Current file-size inventory for `navi-wasm`.

## Rust files over 800 lines outside `src/network/`

- none

## Repo files over 800 lines

Excludes generated output and lockfiles:

- skipped: `target/`, `pkg/`, `node_modules/`, `dist/`, `.git/`
- skipped: `Cargo.lock`

- `demo/tree/tree.js`: 1606 lines
- `demo/level-ordered-tree/level-ordered-tree.js`: 1597 lines
- `demo/network/network.js`: 1305 lines

## Recently completed

- `crates/navi_plot_wasm/src/lib.rs`: split into `crates/navi_plot_wasm/src/wasm_impl/`
- `crates/navi_plot_core/src/tree.rs`: split into `crates/navi_plot_core/src/tree/`
- `crates/navi_plot_core/src/network/tests.rs`: split into `crates/navi_plot_core/src/network/tests/`
- `crates/navi_plot_core/src/network/layout.rs`: split into `crates/navi_plot_core/src/network/layout/`
- `crates/navi_plot_core/src/network/render.rs`: split into `crates/navi_plot_core/src/network/render/`
- `demo/main.js`: split into `demo/main/`

## Next obvious refactor targets

- `demo/tree/tree.js`
- `demo/level-ordered-tree/level-ordered-tree.js`
- `demo/network/network.js`
