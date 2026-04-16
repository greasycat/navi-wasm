import {
  scatterCanvas,
  scatterDetails,
  scatterMetrics,
  scatterReset,
  scatterStressButtons
} from "./dom.js";
import {
  compactStressLabel,
  generateStressScatterSpec,
  initialScatterSpec
} from "./specs.js";
import { state } from "./state.js";
import { setStatus } from "./status.js";
import {
  cloneSpec,
  eventPoint,
  renderProperties,
  selectionName,
  setCanvasDragging
} from "./shared.js";

function setScatterControlsDisabled(disabled) {
  scatterReset.disabled = disabled;
  for (const button of scatterStressButtons) {
    button.disabled = disabled;
  }
}

function supportsScatterSessions(wasm) {
  return typeof wasm.create_scatter_session === "function";
}

export function destroyScatterSession(wasm) {
  if (state.scatterRenderFrame != null) {
    cancelAnimationFrame(state.scatterRenderFrame);
    state.scatterRenderFrame = null;
  }

  if (state.scatterSessionHandle == null || !supportsScatterSessions(wasm)) {
    state.scatterSessionHandle = null;
    return;
  }

  try {
    wasm.destroy_scatter_session(state.scatterSessionHandle);
  } finally {
    state.scatterSessionHandle = null;
  }
}

export function syncScatterSession(wasm) {
  if (!supportsScatterSessions(wasm)) {
    state.scatterSessionHandle = null;
    return;
  }

  destroyScatterSession(wasm);
  state.scatterSessionHandle = wasm.create_scatter_session("scatter-canvas", state.scatterSpec);
}

function renderScatterMetrics() {
  scatterMetrics.textContent =
    `${state.scatterPerf.mode} dataset · ${state.scatterPerf.pointCount.toLocaleString()} points · last scatter render ${state.scatterPerf.renderMs.toFixed(1)} ms`;
}

export function renderScatterOnly(wasm) {
  const start = performance.now();
  if (supportsScatterSessions(wasm)) {
    if (state.scatterSessionHandle == null) {
      syncScatterSession(wasm);
    }
    wasm.render_scatter_session(state.scatterSessionHandle);
  } else {
    wasm.render_scatter("scatter-canvas", state.scatterSpec);
  }

  state.scatterPerf.pointCount = state.scatterSpec.points.length;
  state.scatterPerf.renderMs = performance.now() - start;
  renderScatterMetrics();
}

function scheduleScatterRender(wasm) {
  if (state.scatterRenderFrame != null) {
    return;
  }

  state.scatterRenderFrame = requestAnimationFrame(() => {
    state.scatterRenderFrame = null;
    renderScatterOnly(wasm);
  });
}

export function renderScatterDetails() {
  const index = state.scatterSpec.selected_point_index;
  if (index == null || index < 0 || index >= state.scatterSpec.points.length) {
    scatterDetails.innerHTML = `
      <h3>Point details</h3>
      <p class="details-empty-copy">Click a scatter point to inspect its name, coordinates, and properties.</p>
    `;
    return;
  }

  const point = state.scatterSpec.points[index];
  scatterDetails.innerHTML = `
    <h3>${selectionName(point, `Point ${index + 1}`)}</h3>
    <p class="details-meta">Label ${point.label ?? "none"} · x ${point.x} · y ${point.y}</p>
    ${renderProperties({
      ...point.properties,
      radius: String(point.radius ?? 5),
      color: point.color ?? "default"
    })}
  `;
}

function setScatterSelection(wasm, index) {
  state.scatterSpec.selected_point_index = index;
  if (supportsScatterSessions(wasm) && state.scatterSessionHandle != null) {
    wasm.set_scatter_selection(state.scatterSessionHandle, index ?? undefined);
  }
}

async function loadStressScatter(wasm, count) {
  const label = compactStressLabel(count);
  setScatterControlsDisabled(true);
  setStatus(
    "Generating stress dataset",
    `Building the ${label} scatter dataset and syncing a new Rust session. Large loads can take a moment.`,
    "info"
  );
  await new Promise((resolve) => requestAnimationFrame(resolve));

  try {
    state.scatterSpec = generateStressScatterSpec(count);
    state.scatterPerf.mode = `stress ${label}`;
    syncScatterSession(wasm);
    renderScatterOnly(wasm);
    renderScatterDetails();
    setStatus(
      "Rendered",
      `Loaded the ${label} scatter dataset. Drag either canvas to pan and click a point or node to inspect it. Scatter render time: ${state.scatterPerf.renderMs.toFixed(1)} ms.`,
      "success"
    );
  } finally {
    setScatterControlsDisabled(false);
  }
}

export function attachScatterInteractions(wasm) {
  const dragState = {
    active: false,
    moved: false,
    start: null,
    last: null
  };

  scatterCanvas.addEventListener("pointerdown", (event) => {
    const point = eventPoint(event, scatterCanvas);
    dragState.active = true;
    dragState.moved = false;
    dragState.start = point;
    dragState.last = point;
    scatterCanvas.setPointerCapture(event.pointerId);
    setCanvasDragging(scatterCanvas, true);
  });

  scatterCanvas.addEventListener("pointermove", (event) => {
    if (!dragState.active) {
      return;
    }

    const point = eventPoint(event, scatterCanvas);
    const dx = point.x - dragState.last.x;
    const dy = point.y - dragState.last.y;
    if (dx === 0 && dy === 0) {
      return;
    }

    dragState.last = point;
    dragState.moved = dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) > 4;
    if (supportsScatterSessions(wasm)) {
      wasm.pan_scatter_session(state.scatterSessionHandle, dx, dy);
    } else {
      state.scatterSpec = wasm.pan_scatter(state.scatterSpec, dx, dy);
    }
    scheduleScatterRender(wasm);
  });

  function finishPointer(event) {
    if (!dragState.active) {
      return;
    }

    const point = eventPoint(event, scatterCanvas);
    const wasClick = !dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) <= 4;

    dragState.active = false;
    setCanvasDragging(scatterCanvas, false);
    if (scatterCanvas.hasPointerCapture(event.pointerId)) {
      scatterCanvas.releasePointerCapture(event.pointerId);
    }

    if (wasClick) {
      const hit = supportsScatterSessions(wasm)
        ? wasm.pick_scatter_point_session(state.scatterSessionHandle, point.x, point.y)
        : wasm.pick_scatter_point(state.scatterSpec, point.x, point.y);
      setScatterSelection(wasm, hit?.index ?? null);
      renderScatterOnly(wasm);
      renderScatterDetails();
    }
  }

  scatterCanvas.addEventListener("pointerup", finishPointer);
  scatterCanvas.addEventListener("pointercancel", finishPointer);

  scatterCanvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const point = eventPoint(event, scatterCanvas);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      if (supportsScatterSessions(wasm)) {
        wasm.zoom_scatter_session(state.scatterSessionHandle, point.x, point.y, factor);
      }
      scheduleScatterRender(wasm);
    },
    { passive: false }
  );

  scatterReset.addEventListener("click", () => {
    state.scatterSpec = cloneSpec(initialScatterSpec);
    state.scatterPerf.mode = "sample";
    syncScatterSession(wasm);
    renderScatterOnly(wasm);
    renderScatterDetails();
  });

  for (const button of scatterStressButtons) {
    button.addEventListener("click", async () => {
      const count = Number(button.dataset.stressCount);
      if (!Number.isFinite(count) || count <= 0) {
        return;
      }

      await loadStressScatter(wasm, count);
    });
  }
}
