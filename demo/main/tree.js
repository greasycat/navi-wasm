import { treeCanvas, treeDetails, treeReset } from "./dom.js";
import { TREE_COLLAPSE_ANIMATION_MS, initialTreeSpec } from "./specs.js";
import { state } from "./state.js";
import {
  cloneSpec,
  eventPoint,
  normalizeViewportOffsets,
  renderProperties,
  selectionName,
  setCanvasDragging
} from "./shared.js";

function supportsTreeSessions(wasm) {
  return typeof wasm.create_tree_session === "function";
}

export function destroyTreeSession(wasm) {
  if (state.treeSessionHandle == null || !supportsTreeSessions(wasm)) {
    state.treeSessionHandle = null;
    return;
  }

  try {
    wasm.destroy_tree_session(state.treeSessionHandle);
  } finally {
    state.treeSessionHandle = null;
  }
}

export function syncTreeSession(wasm) {
  if (!supportsTreeSessions(wasm)) {
    state.treeSessionHandle = null;
    return;
  }

  normalizeViewportOffsets(state.treeSpec);
  destroyTreeSession(wasm);
  state.treeSessionHandle = wasm.create_tree_session("tree-canvas", state.treeSpec);
}

function cancelTreeTransition(wasm, renderFinal = false) {
  if (state.treeTransitionFrame != null) {
    cancelAnimationFrame(state.treeTransitionFrame);
    state.treeTransitionFrame = null;
  }

  if (!renderFinal) {
    return;
  }

  if (supportsTreeSessions(wasm)) {
    if (state.treeSessionHandle == null) {
      syncTreeSession(wasm);
    }
    wasm.render_tree_session(state.treeSessionHandle);
    return;
  }

  wasm.render_tree("tree-canvas", state.treeSpec);
}

export function renderTreeOnly(wasm) {
  cancelTreeTransition(wasm, false);
  if (supportsTreeSessions(wasm)) {
    if (state.treeSessionHandle == null) {
      syncTreeSession(wasm);
    }
    wasm.render_tree_session(state.treeSessionHandle);
  } else {
    wasm.render_tree("tree-canvas", state.treeSpec);
  }
}

function buildTreeAdjacency(spec) {
  const children = new Map();
  const parent = new Map();

  for (const node of spec.nodes) {
    children.set(node.id, []);
  }

  for (const edge of spec.edges) {
    children.get(edge.source)?.push(edge.target);
    parent.set(edge.target, edge.source);
  }

  return { children, parent };
}

function countTreeDescendants(spec, nodeId) {
  const { children } = buildTreeAdjacency(spec);
  let count = 0;
  const queue = [...(children.get(nodeId) ?? [])];

  while (queue.length) {
    const current = queue.shift();
    count += 1;
    queue.push(...(children.get(current) ?? []));
  }

  return count;
}

function treeNodeHasChildren(spec, nodeId) {
  const { children } = buildTreeAdjacency(spec);
  return (children.get(nodeId) ?? []).length > 0;
}

function isTreeDescendant(spec, ancestorId, nodeId) {
  if (!ancestorId || !nodeId || ancestorId === nodeId) {
    return false;
  }

  const { parent } = buildTreeAdjacency(spec);
  let cursor = parent.get(nodeId) ?? null;
  while (cursor) {
    if (cursor === ancestorId) {
      return true;
    }
    cursor = parent.get(cursor) ?? null;
  }

  return false;
}

function setTreeNodeCollapsedState(nodeId, collapsed) {
  const next = new Set(state.treeSpec.collapsed_node_ids ?? []);
  if (collapsed) {
    next.add(nodeId);
  } else {
    next.delete(nodeId);
  }
  state.treeSpec.collapsed_node_ids = [...next];
}

function pickTreeNodeAt(wasm, point) {
  if (supportsTreeSessions(wasm)) {
    if (state.treeSessionHandle == null) {
      syncTreeSession(wasm);
    }
    return wasm.pick_tree_node_session(state.treeSessionHandle, point.x, point.y);
  }

  return wasm.pick_tree_node(state.treeSpec, point.x, point.y);
}

function toggleTreeNodeCollapsed(wasm, nodeId) {
  let collapsed = false;

  if (supportsTreeSessions(wasm)) {
    if (state.treeSessionHandle == null) {
      syncTreeSession(wasm);
    }
    collapsed = wasm.toggle_tree_node_collapsed_session(state.treeSessionHandle, nodeId);
  } else {
    if (!treeNodeHasChildren(state.treeSpec, nodeId)) {
      return false;
    }
    const current = new Set(state.treeSpec.collapsed_node_ids ?? []);
    collapsed = !current.has(nodeId);
  }

  setTreeNodeCollapsedState(nodeId, collapsed);
  if (
    collapsed &&
    state.treeSpec.selected_node_id &&
    state.treeSpec.selected_node_id !== nodeId &&
    isTreeDescendant(state.treeSpec, nodeId, state.treeSpec.selected_node_id)
  ) {
    state.treeSpec.selected_node_id = nodeId;
  }

  return collapsed;
}

function easeTreeTransition(progress) {
  if (progress <= 0) {
    return 0;
  }
  if (progress >= 1) {
    return 1;
  }

  return progress < 0.5
    ? 4 * progress * progress * progress
    : 1 - Math.pow(-2 * progress + 2, 3) / 2;
}

function animateTreeCollapse(wasm, nodeId) {
  if (!treeNodeHasChildren(state.treeSpec, nodeId)) {
    return;
  }

  cancelTreeTransition(wasm, false);
  toggleTreeNodeCollapsed(wasm, nodeId);
  renderTreeDetails();

  if (
    !supportsTreeSessions(wasm) ||
    state.treeSessionHandle == null ||
    typeof wasm.render_tree_session_transition !== "function"
  ) {
    renderTreeOnly(wasm);
    return;
  }

  const start = performance.now();
  const step = (now) => {
    const progress = Math.min(1, (now - start) / TREE_COLLAPSE_ANIMATION_MS);
    wasm.render_tree_session_transition(
      state.treeSessionHandle,
      easeTreeTransition(progress)
    );

    if (progress < 1) {
      state.treeTransitionFrame = requestAnimationFrame(step);
      return;
    }

    state.treeTransitionFrame = null;
    renderTreeOnly(wasm);
  };

  state.treeTransitionFrame = requestAnimationFrame(step);
}

export function renderTreeDetails() {
  const nodeId = state.treeSpec.selected_node_id;
  const node = state.treeSpec.nodes.find((entry) => entry.id === nodeId);

  if (!node) {
    treeDetails.innerHTML = `
      <h3>Node details</h3>
      <p class="details-empty-copy">Click a tree node to inspect its name, hierarchy role, and properties.</p>
    `;
    return;
  }

  const { children } = buildTreeAdjacency(state.treeSpec);
  const childCount = (children.get(node.id) ?? []).length;
  const descendantCount = countTreeDescendants(state.treeSpec, node.id);
  const collapsed = (state.treeSpec.collapsed_node_ids ?? []).includes(node.id);
  treeDetails.innerHTML = `
    <h3>${selectionName(node, node.id)}</h3>
    <p class="details-meta">ID ${node.id} · label ${node.label} · children ${childCount} · descendants ${descendantCount} · ${collapsed ? "collapsed" : "expanded"}</p>
    ${renderProperties({
      ...node.properties,
      color: node.color ?? "default"
    })}
  `;
}

function setTreeSelection(wasm, nodeId) {
  state.treeSpec.selected_node_id = nodeId;
  if (supportsTreeSessions(wasm) && state.treeSessionHandle != null) {
    wasm.set_tree_selection(state.treeSessionHandle, nodeId ?? undefined);
  }
}

export function attachTreeInteractions(wasm) {
  const dragState = {
    active: false,
    moved: false,
    start: null,
    last: null
  };

  treeCanvas.addEventListener("pointerdown", (event) => {
    cancelTreeTransition(wasm, true);
    const point = eventPoint(event, treeCanvas);
    dragState.active = true;
    dragState.moved = false;
    dragState.start = point;
    dragState.last = point;
    treeCanvas.setPointerCapture(event.pointerId);
    setCanvasDragging(treeCanvas, true);
  });

  treeCanvas.addEventListener("pointermove", (event) => {
    if (!dragState.active) {
      return;
    }

    const point = eventPoint(event, treeCanvas);
    const dx = point.x - dragState.last.x;
    const dy = point.y - dragState.last.y;
    if (dx === 0 && dy === 0) {
      return;
    }

    dragState.last = point;
    dragState.moved = dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) > 4;
    if (supportsTreeSessions(wasm)) {
      wasm.pan_tree_session(state.treeSessionHandle, dx, dy);
    } else {
      state.treeSpec = wasm.pan_tree(state.treeSpec, dx, dy);
    }
    renderTreeOnly(wasm);
  });

  function finishPointer(event) {
    if (!dragState.active) {
      return;
    }

    const point = eventPoint(event, treeCanvas);
    const wasClick = !dragState.moved || Math.hypot(point.x - dragState.start.x, point.y - dragState.start.y) <= 4;

    dragState.active = false;
    setCanvasDragging(treeCanvas, false);
    if (treeCanvas.hasPointerCapture(event.pointerId)) {
      treeCanvas.releasePointerCapture(event.pointerId);
    }

    if (wasClick) {
      const hit = pickTreeNodeAt(wasm, point);
      setTreeSelection(wasm, hit?.node_id ?? null);
      renderTreeOnly(wasm);
      renderTreeDetails();
    }
  }

  treeCanvas.addEventListener("pointerup", finishPointer);
  treeCanvas.addEventListener("pointercancel", finishPointer);
  treeCanvas.addEventListener("dblclick", (event) => {
    event.preventDefault();
    const point = eventPoint(event, treeCanvas);
    const hit = pickTreeNodeAt(wasm, point);
    if (!hit?.node_id) {
      return;
    }

    animateTreeCollapse(wasm, hit.node_id);
  });

  treeCanvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      cancelTreeTransition(wasm, true);
      const point = eventPoint(event, treeCanvas);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      if (supportsTreeSessions(wasm)) {
        wasm.zoom_tree_session(state.treeSessionHandle, point.x, point.y, factor);
      }
      renderTreeOnly(wasm);
    },
    { passive: false }
  );

  treeReset.addEventListener("click", () => {
    cancelTreeTransition(wasm, false);
    state.treeSpec = cloneSpec(initialTreeSpec);
    syncTreeSession(wasm);
    renderTreeOnly(wasm);
    renderTreeDetails();
  });
}
