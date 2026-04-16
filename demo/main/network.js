import { networkCanvas, networkDetails, networkReset } from "./dom.js";
import { initialNetworkSpec } from "./specs.js";
import { state } from "./state.js";
import {
  buildPropertyList,
  cloneSpec,
  eventPoint,
  normalizeViewportOffsets,
  setCanvasDragging
} from "./shared.js";

function supportsNetworkSessions(wasm) {
  return typeof wasm.create_network_session === "function";
}

export function destroyNetworkSession(wasm) {
  if (state.networkSessionHandle == null || !supportsNetworkSessions(wasm)) {
    state.networkSessionHandle = null;
    return;
  }
  try {
    wasm.destroy_network_session(state.networkSessionHandle);
  } finally {
    state.networkSessionHandle = null;
  }
}

export function syncNetworkSession(wasm) {
  if (!supportsNetworkSessions(wasm)) {
    state.networkSessionHandle = null;
    return;
  }

  normalizeViewportOffsets(state.networkSpec);
  destroyNetworkSession(wasm);
  state.networkSessionHandle = wasm.create_network_session("network-canvas", state.networkSpec);
}

export function renderNetworkOnly(wasm) {
  if (supportsNetworkSessions(wasm)) {
    if (state.networkSessionHandle == null) {
      syncNetworkSession(wasm);
    }
    wasm.render_network_session(state.networkSessionHandle);
  } else {
    wasm.render_network("network-canvas", state.networkSpec);
  }
}

export function renderNetworkDetails() {
  const nodeId = state.networkSpec.selected_node_id;
  const node = state.networkSpec.nodes.find((n) => n.id === nodeId);
  if (!node) {
    const heading = document.createElement("h3");
    heading.textContent = "Node details";
    const body = document.createElement("p");
    body.className = "details-empty-copy";
    body.textContent = "Click a node to inspect its ID, connections, and properties.";
    networkDetails.replaceChildren(heading, body);
    return;
  }
  const outgoing = state.networkSpec.edges.filter((e) => e.source === node.id).length;
  const incoming = state.networkSpec.edges.filter((e) => e.target === node.id).length;
  const heading = document.createElement("h3");
  heading.textContent = node.label ?? node.id;
  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = `ID ${node.id} · in ${incoming} · out ${outgoing}`;
  networkDetails.replaceChildren(heading, meta, buildPropertyList({ ...node.properties, color: node.color ?? "default" }));
}

function setNetworkSelection(wasm, nodeId) {
  state.networkSpec.selected_node_id = nodeId;
  if (supportsNetworkSessions(wasm) && state.networkSessionHandle != null) {
    wasm.set_network_selection(state.networkSessionHandle, nodeId != null ? nodeId : undefined);
  }
}

export function attachNetworkInteractions(wasm) {
  const dragState = { active: false, moved: false, start: null, last: null };

  networkCanvas.addEventListener("pointerdown", (event) => {
    const point = eventPoint(event, networkCanvas);
    dragState.active = true;
    dragState.moved = false;
    dragState.start = point;
    dragState.last = point;
    networkCanvas.setPointerCapture(event.pointerId);
    setCanvasDragging(networkCanvas, true);
  });

  networkCanvas.addEventListener("pointermove", (event) => {
    if (!dragState.active) return;
    const point = eventPoint(event, networkCanvas);
    const dx = point.x - dragState.last.x;
    const dy = point.y - dragState.last.y;
    if (dx === 0 && dy === 0) return;
    dragState.last = point;
    dragState.moved = dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) > 4;
    if (supportsNetworkSessions(wasm)) {
      wasm.pan_network_session(state.networkSessionHandle, dx, dy);
    } else {
      state.networkSpec = wasm.pan_network(state.networkSpec, dx, dy);
    }
    renderNetworkOnly(wasm);
  });

  function finishNetworkPointer(event) {
    if (!dragState.active) return;
    const point = eventPoint(event, networkCanvas);
    const wasClick = !dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) <= 4;
    dragState.active = false;
    setCanvasDragging(networkCanvas, false);
    if (networkCanvas.hasPointerCapture(event.pointerId)) {
      networkCanvas.releasePointerCapture(event.pointerId);
    }
    if (wasClick) {
      const hit = supportsNetworkSessions(wasm)
        ? wasm.pick_network_node_session(state.networkSessionHandle, point.x, point.y)
        : wasm.pick_network_node(state.networkSpec, point.x, point.y);
      setNetworkSelection(wasm, hit?.node_id ?? null);
      renderNetworkOnly(wasm);
      renderNetworkDetails();
    }
  }

  networkCanvas.addEventListener("pointerup", finishNetworkPointer);
  networkCanvas.addEventListener("pointercancel", finishNetworkPointer);

  networkCanvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const point = eventPoint(event, networkCanvas);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      if (supportsNetworkSessions(wasm)) {
        wasm.zoom_network_session(state.networkSessionHandle, point.x, point.y, factor);
      }
      renderNetworkOnly(wasm);
    },
    { passive: false }
  );

  networkReset.addEventListener("click", () => {
    state.networkSpec = cloneSpec(initialNetworkSpec);
    syncNetworkSession(wasm);
    renderNetworkOnly(wasm);
    renderNetworkDetails();
  });
}
