import {
  barCanvas,
  barDetails,
  barGroupedButton,
  barReset,
  barStackedButton
} from "./dom.js";
import { initialBarSpec } from "./specs.js";
import { state } from "./state.js";
import { buildPropertyList, cloneSpec, eventPoint } from "./shared.js";

function supportsBarSessions(wasm) {
  return typeof wasm.create_bar_session === "function";
}

export function destroyBarSession(wasm) {
  if (state.barSessionHandle == null || !supportsBarSessions(wasm)) {
    state.barSessionHandle = null;
    return;
  }

  try {
    wasm.destroy_bar_session(state.barSessionHandle);
  } finally {
    state.barSessionHandle = null;
  }
}

export function syncBarSession(wasm) {
  if (!supportsBarSessions(wasm)) {
    state.barSessionHandle = null;
    return;
  }

  destroyBarSession(wasm);
  state.barSessionHandle = wasm.create_bar_session("bar-canvas", state.barSpec);
}

export function renderBarOnly(wasm) {
  if (supportsBarSessions(wasm)) {
    if (state.barSessionHandle == null) {
      syncBarSession(wasm);
    }
    wasm.render_bar_session(state.barSessionHandle);
  } else {
    wasm.render_bar("bar-canvas", state.barSpec);
  }
}

export function renderBarDetails() {
  const sel = state.barSpec.selected_bar;
  if (!sel) {
    const heading = document.createElement("h3");
    heading.textContent = "Bar details";
    const body = document.createElement("p");
    body.className = "details-empty-copy";
    body.textContent = "Click a bar to inspect its series, category, and value.";
    barDetails.replaceChildren(heading, body);
    return;
  }
  const [si, ci] = sel;
  const series = state.barSpec.series[si];
  const category = state.barSpec.categories[ci];
  if (!series || category == null) {
    const heading = document.createElement("h3");
    heading.textContent = "Bar details";
    barDetails.replaceChildren(heading);
    return;
  }
  const heading = document.createElement("h3");
  heading.textContent = `${series.label} · ${category}`;
  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = `Series ${si + 1} · Category ${ci + 1} · Value ${series.values[ci]}`;
  barDetails.replaceChildren(heading, meta, buildPropertyList({ series: series.label, category, value: String(series.values[ci]) }));
}

function setBarSelection(wasm, seriesIndex, categoryIndex) {
  state.barSpec.selected_bar =
    seriesIndex != null && categoryIndex != null ? [seriesIndex, categoryIndex] : null;
  if (supportsBarSessions(wasm) && state.barSessionHandle != null) {
    wasm.set_bar_selection(
      state.barSessionHandle,
      seriesIndex != null ? seriesIndex : undefined,
      categoryIndex != null ? categoryIndex : undefined
    );
  }
}

export function attachBarInteractions(wasm) {
  barCanvas.addEventListener("click", (event) => {
    const point = eventPoint(event, barCanvas);
    const hit = supportsBarSessions(wasm)
      ? wasm.pick_bar_session(state.barSessionHandle, point.x, point.y)
      : wasm.pick_bar(state.barSpec, point.x, point.y);
    setBarSelection(wasm, hit?.series_index ?? null, hit?.category_index ?? null);
    renderBarOnly(wasm);
    renderBarDetails();
  });

  barCanvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const point = eventPoint(event, barCanvas);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      if (supportsBarSessions(wasm)) {
        wasm.zoom_bar_session(state.barSessionHandle, point.x, point.y, factor);
      }
      renderBarOnly(wasm);
    },
    { passive: false }
  );

  barGroupedButton.addEventListener("click", () => {
    state.barSpec.variant = "grouped";
    syncBarSession(wasm);
    renderBarOnly(wasm);
    renderBarDetails();
  });

  barStackedButton.addEventListener("click", () => {
    state.barSpec.variant = "stacked";
    syncBarSession(wasm);
    renderBarOnly(wasm);
    renderBarDetails();
  });

  barReset.addEventListener("click", () => {
    state.barSpec = cloneSpec(initialBarSpec);
    syncBarSession(wasm);
    renderBarOnly(wasm);
    renderBarDetails();
  });
}
