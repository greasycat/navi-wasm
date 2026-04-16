import { attachBarInteractions, destroyBarSession, renderBarDetails, renderBarOnly, syncBarSession } from "./bar.js";
import { attachHeatmapInteractions, destroyHeatmapSession, renderHeatmapDetails, renderHeatmapOnly, syncHeatmapSession } from "./heatmap.js";
import { attachLineInteractions, destroyLineSession, renderLineDetails, renderLineOnly, syncLineSession } from "./line.js";
import { attachNetworkInteractions, destroyNetworkSession, renderNetworkDetails, renderNetworkOnly, syncNetworkSession } from "./network.js";
import { attachScatterInteractions, destroyScatterSession, renderScatterDetails, renderScatterOnly, syncScatterSession } from "./scatter.js";
import { state } from "./state.js";
import { setStatus } from "./status.js";
import { attachTreeInteractions, destroyTreeSession, renderTreeDetails, renderTreeOnly, syncTreeSession } from "./tree.js";

async function loadWasm() {
  const moduleUrl = new URL("../../pkg/navi_plot_wasm.js", import.meta.url);
  const wasm = await import(moduleUrl.href);
  await wasm.default();
  return wasm;
}

function renderInitialScene(wasm) {
  syncScatterSession(wasm);
  renderScatterOnly(wasm);
  syncTreeSession(wasm);
  renderTreeOnly(wasm);
  syncLineSession(wasm);
  renderLineOnly(wasm);
  syncBarSession(wasm);
  renderBarOnly(wasm);
  syncHeatmapSession(wasm);
  renderHeatmapOnly(wasm);
  syncNetworkSession(wasm);
  renderNetworkOnly(wasm);
  renderScatterDetails();
  renderTreeDetails();
  renderLineDetails();
  renderBarDetails();
  renderHeatmapDetails();
  renderNetworkDetails();
  setStatus(
    "Rendered",
    `The wasm package loaded successfully. Drag pannable canvases to pan and click elements to inspect them. Scatter render time: ${state.scatterPerf.renderMs.toFixed(1)} ms.`,
    "success"
  );
}

function bootInteractiveDemo(wasm) {
  renderInitialScene(wasm);
  attachScatterInteractions(wasm);
  attachTreeInteractions(wasm);
  attachLineInteractions(wasm);
  attachBarInteractions(wasm);
  attachHeatmapInteractions(wasm);
  attachNetworkInteractions(wasm);
  window.addEventListener("beforeunload", () => {
    destroyScatterSession(wasm);
    destroyTreeSession(wasm);
    destroyLineSession(wasm);
    destroyBarSession(wasm);
    destroyHeatmapSession(wasm);
    destroyNetworkSession(wasm);
  }, { once: true });
}

export async function startDemo() {
  try {
    const wasm = await loadWasm();
    bootInteractiveDemo(wasm);
  } catch (error) {
    console.error(error);
    setStatus(
      "WASM package not found",
      "Build the package from the workspace root with <code>wasm-pack build crates/navi_plot_wasm --target web --out-dir ../../pkg</code>, then serve the repository root and open <code>/demo/</code>.",
      "error"
    );
  }
}
