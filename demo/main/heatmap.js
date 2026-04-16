import {
  heatmapBwrButton,
  heatmapCanvas,
  heatmapDetails,
  heatmapGreensButton,
  heatmapViridisButton
} from "./dom.js";
import { initialHeatmapSpec } from "./specs.js";
import { state } from "./state.js";
import { buildPropertyList, eventPoint } from "./shared.js";

function supportsHeatmapSessions(wasm) {
  return typeof wasm.create_heatmap_session === "function";
}

export function destroyHeatmapSession(wasm) {
  if (state.heatmapSessionHandle == null || !supportsHeatmapSessions(wasm)) {
    state.heatmapSessionHandle = null;
    return;
  }

  try {
    wasm.destroy_heatmap_session(state.heatmapSessionHandle);
  } finally {
    state.heatmapSessionHandle = null;
  }
}

export function syncHeatmapSession(wasm) {
  if (!supportsHeatmapSessions(wasm)) {
    state.heatmapSessionHandle = null;
    return;
  }

  destroyHeatmapSession(wasm);
  state.heatmapSessionHandle = wasm.create_heatmap_session("heatmap-canvas", state.heatmapSpec);
}

export function renderHeatmapOnly(wasm) {
  if (supportsHeatmapSessions(wasm)) {
    if (state.heatmapSessionHandle == null) {
      syncHeatmapSession(wasm);
    }
    wasm.render_heatmap_session(state.heatmapSessionHandle);
  } else {
    wasm.render_heatmap("heatmap-canvas", state.heatmapSpec);
  }
}

export function renderHeatmapDetails() {
  const sel = state.heatmapSpec.selected_cell;
  if (!sel) {
    const heading = document.createElement("h3");
    heading.textContent = "Cell details";
    const body = document.createElement("p");
    body.className = "details-empty-copy";
    body.textContent = "Click a cell to inspect its row, column, and value.";
    heatmapDetails.replaceChildren(heading, body);
    return;
  }
  const [row, col] = sel;
  const rowLabel = state.heatmapSpec.row_labels[row] ?? `Row ${row}`;
  const colLabel = state.heatmapSpec.col_labels[col] ?? `Col ${col}`;
  const value = state.heatmapSpec.cells[row]?.[col];
  if (value == null) {
    const heading = document.createElement("h3");
    heading.textContent = "Cell details";
    heatmapDetails.replaceChildren(heading);
    return;
  }
  const heading = document.createElement("h3");
  heading.textContent = `${rowLabel} → ${colLabel}`;
  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = `Row ${row} · Col ${col} · Value ${value}`;
  heatmapDetails.replaceChildren(heading, meta, buildPropertyList({ row: rowLabel, column: colLabel, value: String(value) }));
}

function setHeatmapSelection(wasm, row, col) {
  state.heatmapSpec.selected_cell = row != null && col != null ? [row, col] : null;
  if (supportsHeatmapSessions(wasm) && state.heatmapSessionHandle != null) {
    wasm.set_heatmap_selection(
      state.heatmapSessionHandle,
      row != null ? row : undefined,
      col != null ? col : undefined
    );
  }
}

export function attachHeatmapInteractions(wasm) {
  heatmapCanvas.addEventListener("click", (event) => {
    const point = eventPoint(event, heatmapCanvas);
    const hit = supportsHeatmapSessions(wasm)
      ? wasm.pick_heatmap_cell_session(state.heatmapSessionHandle, point.x, point.y)
      : wasm.pick_heatmap_cell(state.heatmapSpec, point.x, point.y);
    setHeatmapSelection(wasm, hit?.row ?? null, hit?.col ?? null);
    renderHeatmapOnly(wasm);
    renderHeatmapDetails();
  });

  heatmapCanvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const point = eventPoint(event, heatmapCanvas);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      if (supportsHeatmapSessions(wasm)) {
        wasm.zoom_heatmap_session(state.heatmapSessionHandle, point.x, point.y, factor);
      }
      renderHeatmapOnly(wasm);
    },
    { passive: false }
  );

  heatmapBwrButton.addEventListener("click", () => {
    state.heatmapSpec.palette = "blue_white_red";
    syncHeatmapSession(wasm);
    renderHeatmapOnly(wasm);
    renderHeatmapDetails();
  });

  heatmapViridisButton.addEventListener("click", () => {
    state.heatmapSpec.palette = "viridis";
    syncHeatmapSession(wasm);
    renderHeatmapOnly(wasm);
    renderHeatmapDetails();
  });

  heatmapGreensButton.addEventListener("click", () => {
    state.heatmapSpec.palette = "greens";
    syncHeatmapSession(wasm);
    renderHeatmapOnly(wasm);
    renderHeatmapDetails();
  });
}
