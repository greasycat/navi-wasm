import { lineCanvas, lineDetails, lineReset } from "./dom.js";
import { initialLineSpec } from "./specs.js";
import { state } from "./state.js";
import { buildPropertyList, cloneSpec, eventPoint, setCanvasDragging } from "./shared.js";

function supportsLineSessions(wasm) {
  return typeof wasm.create_line_session === "function";
}

export function destroyLineSession(wasm) {
  if (state.lineSessionHandle == null || !supportsLineSessions(wasm)) {
    state.lineSessionHandle = null;
    return;
  }
  try {
    wasm.destroy_line_session(state.lineSessionHandle);
  } finally {
    state.lineSessionHandle = null;
  }
}

export function syncLineSession(wasm) {
  if (!supportsLineSessions(wasm)) {
    state.lineSessionHandle = null;
    return;
  }
  destroyLineSession(wasm);
  state.lineSessionHandle = wasm.create_line_session("line-canvas", state.lineSpec);
}

export function renderLineOnly(wasm) {
  if (supportsLineSessions(wasm)) {
    if (state.lineSessionHandle == null) {
      syncLineSession(wasm);
    }
    wasm.render_line_session(state.lineSessionHandle);
  } else {
    wasm.render_line("line-canvas", state.lineSpec);
  }
}

export function renderLineDetails() {
  const sel = state.lineSpec.selected_point;
  if (!sel) {
    const heading = document.createElement("h3");
    heading.textContent = "Point details";
    const body = document.createElement("p");
    body.className = "details-empty-copy";
    body.textContent = "Click a point to inspect its series, coordinates, and properties.";
    lineDetails.replaceChildren(heading, body);
    return;
  }
  const [si, pi] = sel;
  const series = state.lineSpec.series[si];
  const point = series?.points[pi];
  if (!point) {
    const heading = document.createElement("h3");
    heading.textContent = "Point details";
    lineDetails.replaceChildren(heading);
    return;
  }
  const heading = document.createElement("h3");
  heading.textContent = point.label ?? `Series ${si + 1} · Point ${pi + 1}`;
  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = `Series "${series.label}" · x ${point.x} · y ${point.y}`;
  lineDetails.replaceChildren(heading, meta, buildPropertyList(point.properties));
}

function setLineSelection(wasm, seriesIndex, pointIndex) {
  state.lineSpec.selected_point = (seriesIndex != null && pointIndex != null) ? [seriesIndex, pointIndex] : null;
  if (supportsLineSessions(wasm) && state.lineSessionHandle != null) {
    wasm.set_line_selection(
      state.lineSessionHandle,
      seriesIndex != null ? seriesIndex : undefined,
      pointIndex != null ? pointIndex : undefined
    );
  }
}

export function attachLineInteractions(wasm) {
  const dragState = { active: false, moved: false, start: null, last: null };

  lineCanvas.addEventListener("pointerdown", (event) => {
    const point = eventPoint(event, lineCanvas);
    dragState.active = true;
    dragState.moved = false;
    dragState.start = point;
    dragState.last = point;
    lineCanvas.setPointerCapture(event.pointerId);
    setCanvasDragging(lineCanvas, true);
  });

  lineCanvas.addEventListener("pointermove", (event) => {
    if (!dragState.active) return;
    const point = eventPoint(event, lineCanvas);
    const dx = point.x - dragState.last.x;
    const dy = point.y - dragState.last.y;
    if (dx === 0 && dy === 0) return;
    dragState.last = point;
    dragState.moved = dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) > 4;
    if (supportsLineSessions(wasm)) {
      wasm.pan_line_session(state.lineSessionHandle, dx, dy);
    } else {
      state.lineSpec = wasm.pan_line(state.lineSpec, dx, dy);
    }
    renderLineOnly(wasm);
  });

  function finishLinePointer(event) {
    if (!dragState.active) return;
    const point = eventPoint(event, lineCanvas);
    const wasClick = !dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) <= 4;
    dragState.active = false;
    setCanvasDragging(lineCanvas, false);
    if (lineCanvas.hasPointerCapture(event.pointerId)) {
      lineCanvas.releasePointerCapture(event.pointerId);
    }
    if (wasClick) {
      const hit = supportsLineSessions(wasm)
        ? wasm.pick_line_point_session(state.lineSessionHandle, point.x, point.y)
        : wasm.pick_line_point(state.lineSpec, point.x, point.y);
      setLineSelection(wasm, hit?.series_index ?? null, hit?.point_index ?? null);
      renderLineOnly(wasm);
      renderLineDetails();
    }
  }

  lineCanvas.addEventListener("pointerup", finishLinePointer);
  lineCanvas.addEventListener("pointercancel", finishLinePointer);

  lineCanvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const point = eventPoint(event, lineCanvas);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      if (supportsLineSessions(wasm)) {
        wasm.zoom_line_session(state.lineSessionHandle, point.x, point.y, factor);
      }
      renderLineOnly(wasm);
    },
    { passive: false }
  );

  lineReset.addEventListener("click", () => {
    state.lineSpec = cloneSpec(initialLineSpec);
    syncLineSession(wasm);
    renderLineOnly(wasm);
    renderLineDetails();
  });
}
