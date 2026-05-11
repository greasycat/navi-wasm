import init, * as wasm from "../../pkg/navi_plot_wasm.js";

const DATA_URL = "./data/ladw-network.json";
const ROOT_NODE_ID = "__start__";
const TOGGLEABLE_PROPERTY_KEY = "navil_toggleable";
const EXPANDED_PROPERTY_KEY = "navil_expanded";
const HIERARCHY_EDGE_WEIGHT = 1.0;
const EXPANDED_PARENT_EDGE_WEIGHT = 0.65;
const SIBLING_EDGE_WEIGHT = 0.15;

const LEVEL_COLORS = ["#1d7a42", "#0891b2", "#7c3aed", "#d97706", "#dc2626"];
const STAGE_COLORS = ["#0f766e", "#2563eb", "#7c3aed", "#d97706", "#a61b3f", "#4f46e5"];
const EXERCISE_COLOR = "#b45309";
const TOPOLOGY_ANIMATION_MS = 220;
const COMPONENT_SUBGRAPH_ID = "chapter-1-vector-spaces-refined";
const COMPONENT_NODE_PREFIX = "cmp-";
const COMPONENT_FORCE_LAYER_OFFSET = 1_000_000_000;
const ATTACHED_CUSTOM_SUBGRAPH_ROLE = "attached_custom";
const SUBGRAPH_LAYOUT = {
  DEMO: "demo",
  NAVIL: "navil",
  NAVIL_STRONG: "navil-strong",
  NAVIL_ANCHOR: "navil-anchor",
  NAVIL_ANCHOR_STRONG: "navil-anchor-strong",
  NAVIL_DEMO_PARAMS: "navil-demo-params"
};
const NAVIL_VARIANTS = {
  [SUBGRAPH_LAYOUT.NAVIL]: {
    componentEdgeWeight: 0.45,
    keepRootAnchor: false,
    useDemoParams: false
  },
  [SUBGRAPH_LAYOUT.NAVIL_STRONG]: {
    componentEdgeWeight: 0.62,
    keepRootAnchor: false,
    useDemoParams: false
  },
  [SUBGRAPH_LAYOUT.NAVIL_ANCHOR]: {
    componentEdgeWeight: 0.45,
    keepRootAnchor: true,
    useDemoParams: false
  },
  [SUBGRAPH_LAYOUT.NAVIL_ANCHOR_STRONG]: {
    componentEdgeWeight: 0.62,
    keepRootAnchor: true,
    useDemoParams: false
  },
  [SUBGRAPH_LAYOUT.NAVIL_DEMO_PARAMS]: {
    componentEdgeWeight: 0.45,
    keepRootAnchor: false,
    useDemoParams: true
  }
};
const ROLE_COLORS = {
  definition: "#2563eb",
  example: "#0f766e",
  question: "#b45309",
  narrative: "#6b7280",
  theorem: "#7c3aed",
  proof: "#a61b3f",
  claim: "#4f46e5",
  caption: "#64748b"
};

const state = {
  data: null,
  entriesByOrder: new Map(),
  childrenByParent: new Map(),
  rankByOrder: new Map(),
  stageByOrder: new Map(),
  componentSubgraphsById: new Map(),
  activeSubgraphId: null,
  activeSubgraphLayout: null,
  expandedIds: new Set(),
  selectedNodeId: null,
  sessionHandle: null,
  transitionFrame: null,
  pixelRatio: Math.max(1, Math.min(3, Math.round((window.devicePixelRatio || 1) * 2) / 2)),
  resizeFrame: null
};

const dom = {
  title: document.getElementById("book-title"),
  status: document.getElementById("status-panel"),
  tocList: document.getElementById("toc-list"),
  visibleCount: document.getElementById("visible-count"),
  canvas: document.getElementById("network-canvas"),
  frame: document.querySelector(".canvas-frame"),
  details: document.getElementById("node-details"),
  reset: document.getElementById("reset-view"),
  expandAll: document.getElementById("expand-all"),
  collapseAll: document.getElementById("collapse-all"),
  loadVectorSubgraph: document.getElementById("load-vector-subgraph"),
  loadNavilSubgraph: document.getElementById("load-navil-subgraph"),
  loadNavilStrongSubgraph: document.getElementById("load-navil-strong-subgraph"),
  loadNavilAnchorSubgraph: document.getElementById("load-navil-anchor-subgraph"),
  loadNavilAnchorStrongSubgraph: document.getElementById("load-navil-anchor-strong-subgraph"),
  loadNavilDemoParamsSubgraph: document.getElementById("load-navil-demo-params-subgraph"),
  wholeGraph: document.getElementById("whole-graph"),
  metricVisible: document.getElementById("metric-visible"),
  metricEdges: document.getElementById("metric-edges"),
  metricPages: document.getElementById("metric-pages"),
  metricStages: document.getElementById("metric-stages")
};

function nodeId(orderIndex) {
  return `toc-${orderIndex}`;
}

function componentNodeId(componentId) {
  return `${COMPONENT_NODE_PREFIX}${componentId}`;
}

function orderIndexFromNodeId(id) {
  if (!id || !id.startsWith("toc-")) return null;
  const parsed = Number.parseInt(id.slice(4), 10);
  return Number.isFinite(parsed) ? parsed : null;
}

function componentIdFromNodeId(id) {
  if (!id || !id.startsWith(COMPONENT_NODE_PREFIX)) return null;
  return id.slice(COMPONENT_NODE_PREFIX.length);
}

function setStatus(kind, title, body) {
  dom.status.className = `status-panel is-visible status-${kind}`;
  dom.status.replaceChildren();
  const heading = document.createElement("p");
  heading.className = "status-title";
  heading.textContent = title;
  const copy = document.createElement("p");
  copy.className = "status-body";
  copy.textContent = body;
  dom.status.append(heading, copy);
}

function clearStatus() {
  dom.status.className = "status-panel";
  dom.status.replaceChildren();
}

function isExerciseEntry(entry) {
  return Boolean(entry.is_problem) || /\b(exercises?|problems?)\b/i.test(entry.title);
}

function levelColor(entry) {
  if (isExerciseEntry(entry)) return EXERCISE_COLOR;
  const idx = Math.max(0, Math.min(LEVEL_COLORS.length - 1, Number(entry.level || 1) - 1));
  return LEVEL_COLORS[idx];
}

function stageColor(orderIndex) {
  const stage = state.stageByOrder.get(orderIndex);
  return stage ? STAGE_COLORS[stage.stageIndex % STAGE_COLORS.length] : null;
}

function nodeColor(entry) {
  return stageColor(entry.order_index) ?? levelColor(entry);
}

function componentColor(component) {
  return ROLE_COLORS[component.role] ?? "#64748b";
}

function scale(value, minimum = 0) {
  if (value == null) return value;
  return Math.max(minimum, Math.round(value * state.pixelRatio));
}

function truncateLabel(title) {
  return title.length > 28 ? `${title.slice(0, 26)}...` : title;
}

function componentLabel(component) {
  const base = component.description || component.role || "component";
  return base.length > 26 ? `${base.slice(0, 24)}...` : base;
}

function roleSummary(entry) {
  return Object.entries(entry.roles ?? {})
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, 5);
}

function normalizedRelevance(entry) {
  const value = Number(entry.normalized_relevance);
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(1, value));
}

function relevanceRadius(entry) {
  return 10 + Math.round(normalizedRelevance(entry) * 18);
}

function formatRelevance(entry) {
  const effective = Number(entry.effective_relevance);
  if (!Number.isFinite(effective)) return "0.00";
  return effective.toFixed(2);
}

function easeInOutCubic(t) {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

function prepareData(data) {
  state.data = data;
  state.entriesByOrder = new Map(data.entries.map((entry) => [entry.order_index, entry]));
  state.childrenByParent = new Map();
  state.rankByOrder = new Map();
  state.stageByOrder = new Map();
  state.componentSubgraphsById = new Map(
    (data.component_subgraphs ?? []).map((subgraph) => [subgraph.id, subgraph])
  );

  for (const entry of data.entries) {
    const parent = entry.parent_order_index ?? ROOT_NODE_ID;
    const bucket = state.childrenByParent.get(parent) ?? [];
    bucket.push(entry);
    state.childrenByParent.set(parent, bucket);
  }

  for (const children of state.childrenByParent.values()) {
    children.sort((a, b) => a.order_index - b.order_index);
  }

  for (const ranked of data.path?.ranked_toc ?? []) {
    state.rankByOrder.set(ranked.order_index, ranked);
  }

  (data.path?.path_stages ?? []).forEach((stage, stageIndex) => {
    for (const orderIndex of stage.toc_order_indices ?? []) {
      if (!state.stageByOrder.has(orderIndex)) {
        state.stageByOrder.set(orderIndex, {
          stageIndex,
          title: stage.title,
          focus: stage.focus,
          rationale: stage.rationale
        });
      }
    }
  });
}

function activeSubgraph() {
  return state.activeSubgraphId ? state.componentSubgraphsById.get(state.activeSubgraphId) ?? null : null;
}

function isNavilLayoutSubgraph() {
  return Boolean(activeNavilVariant());
}

function activeNavilVariant() {
  return activeSubgraph() ? NAVIL_VARIANTS[state.activeSubgraphLayout] ?? null : null;
}

function topLevelEntries() {
  const subgraph = activeSubgraph();
  if (subgraph) {
    const root = state.entriesByOrder.get(subgraph.root_order_index);
    return root ? [root] : [];
  }
  return state.childrenByParent.get(ROOT_NODE_ID) ?? [];
}

function initialExpandedIds() {
  const subgraph = activeSubgraph();
  if (subgraph) {
    return new Set(
      subgraph.toc_order_indices
        .map((orderIndex) => state.entriesByOrder.get(orderIndex))
        .filter((entry) => entry && hasChildren(entry))
        .map((entry) => nodeId(entry.order_index))
    );
  }
  return new Set(topLevelEntries().filter((entry) => entry.child_count > 0).map((entry) => nodeId(entry.order_index)));
}

function findInitialSelection() {
  return (
    state.data.entries.find((entry) => /^chapter\b/i.test(entry.title)) ??
    state.data.entries.find((entry) => entry.parent_order_index == null) ??
    state.data.entries[0] ??
    null
  );
}

function entryChildren(entry) {
  return state.childrenByParent.get(entry.order_index) ?? [];
}

function hasChildren(entry) {
  return entryChildren(entry).length > 0;
}

function visibleEntries() {
  const rows = [];

  function walk(entries, depth) {
    for (const entry of entries) {
      rows.push({ entry, depth });
      if (state.expandedIds.has(nodeId(entry.order_index))) {
        walk(entryChildren(entry), depth + 1);
      }
    }
  }

  walk(topLevelEntries(), 0);
  return rows;
}

function activeComponents() {
  return activeSubgraph()?.components ?? [];
}

function findComponent(componentId) {
  return activeComponents().find((component) => component.id === componentId) ?? null;
}

function ancestorOrderIndices(entry) {
  const ancestors = [];
  let parent = entry.parent_order_index;
  while (parent != null) {
    ancestors.push(parent);
    parent = state.entriesByOrder.get(parent)?.parent_order_index ?? null;
  }
  return ancestors;
}

function expandAncestors(entry) {
  for (const orderIndex of ancestorOrderIndices(entry)) {
    const ancestor = state.entriesByOrder.get(orderIndex);
    if (ancestor && hasChildren(ancestor)) {
      state.expandedIds.add(nodeId(orderIndex));
    }
  }
}

function isDescendantOf(candidate, ancestorOrderIndex) {
  let parent = candidate.parent_order_index;
  while (parent != null) {
    if (parent === ancestorOrderIndex) return true;
    parent = state.entriesByOrder.get(parent)?.parent_order_index ?? null;
  }
  return false;
}

function forceLayers(entry) {
  if (!entry.path) return [entry.order_index];
  const layers = entry.path
    .split(".")
    .map((part) => Number.parseInt(part.replace(/^n/, ""), 10))
    .filter((value) => Number.isFinite(value));
  return layers.length > 0 ? layers : [entry.order_index];
}

function descendantEntries(entry) {
  const descendants = [];

  function walk(current) {
    descendants.push(current);
    for (const child of entryChildren(current)) {
      walk(child);
    }
  }

  walk(entry);
  return descendants;
}

function stageFilteredOrderSet(subgraph) {
  const stageIndex = state.stageByOrder.get(subgraph.root_order_index)?.stageIndex;
  const stage = Number.isInteger(stageIndex) ? state.data.path?.path_stages?.[stageIndex] : null;
  if (!stage) {
    return new Set(state.data.entries.map((entry) => entry.order_index));
  }

  const included = new Set();
  for (const orderIndex of stage.toc_order_indices ?? []) {
    const entry = state.entriesByOrder.get(orderIndex);
    if (!entry) continue;
    for (const descendant of descendantEntries(entry)) {
      included.add(descendant.order_index);
    }
    included.add(entry.order_index);
    for (const ancestorOrderIndex of ancestorOrderIndices(entry)) {
      included.add(ancestorOrderIndex);
    }
  }

  return included;
}

function topEntriesForOrderSet(orderSet) {
  return state.data.entries
    .filter((entry) => orderSet.has(entry.order_index))
    .filter((entry) => entry.parent_order_index == null || !orderSet.has(entry.parent_order_index))
    .sort((a, b) => a.order_index - b.order_index);
}

function visibleEntriesFromTopEntries(entries, allowedOrderSet = null) {
  const rows = [];

  function walk(items, depth) {
    for (const entry of items) {
      if (allowedOrderSet && !allowedOrderSet.has(entry.order_index)) continue;
      rows.push({ entry, depth });
      if (state.expandedIds.has(nodeId(entry.order_index))) {
        walk(entryChildren(entry), depth + 1);
      }
    }
  }

  walk(entries, 0);
  return rows;
}

function collectVisibleSubtreeNodeIds(rootEntry, visibleNodeIds) {
  const ids = [];

  function walk(entry) {
    const id = nodeId(entry.order_index);
    if (!visibleNodeIds.has(id)) return;
    ids.push(id);
    for (const child of entryChildren(entry)) {
      walk(child);
    }
  }

  walk(rootEntry);
  return ids;
}

function buildAttachedSubgraphNodeIds(spec, rootEntry, { keepRootAnchor = false } = {}) {
  const nodeById = new Map(spec.nodes.map((node) => [node.id, node]));
  const included = new Set(collectVisibleSubtreeNodeIds(rootEntry, nodeById));
  if (keepRootAnchor && nodeById.has(ROOT_NODE_ID)) {
    included.add(ROOT_NODE_ID);
  }
  let changed = true;

  while (changed) {
    changed = false;
    for (const edge of spec.edges) {
      if (included.has(edge.source)) {
        const target = nodeById.get(edge.target);
        if (target?.properties?.navil_subgraph_role === ATTACHED_CUSTOM_SUBGRAPH_ROLE && !included.has(target.id)) {
          included.add(target.id);
          changed = true;
        }
      }
      if (included.has(edge.target)) {
        const source = nodeById.get(edge.source);
        if (source?.properties?.navil_subgraph_role === ATTACHED_CUSTOM_SUBGRAPH_ROLE && !included.has(source.id)) {
          included.add(source.id);
          changed = true;
        }
      }
    }
  }

  return spec.nodes.map((node) => node.id).filter((id) => included.has(id));
}

function inducedSubgraph(spec, includedNodeIds, { rootEntry = null, keepRootAnchor = false } = {}) {
  const included = new Set(includedNodeIds);
  const nodes = spec.nodes.filter((node) => included.has(node.id));
  const edges = spec.edges.filter((edge) => included.has(edge.source) && included.has(edge.target));
  const rootTargetId = rootEntry ? nodeId(rootEntry.order_index) : null;
  if (
    keepRootAnchor &&
    rootTargetId &&
    included.has(ROOT_NODE_ID) &&
    included.has(rootTargetId) &&
    !edges.some((edge) => edge.source === ROOT_NODE_ID && edge.target === rootTargetId)
  ) {
    edges.push({
      source: ROOT_NODE_ID,
      target: rootTargetId,
      weight: HIERARCHY_EDGE_WEIGHT,
      style: { stroke_width: 0, opacity: 0 }
    });
  }

  return {
    ...spec,
    nodes,
    edges,
    selected_node_id: included.has(spec.selected_node_id) ? spec.selected_node_id : undefined
  };
}

function structuralStats(spec) {
  const ids = new Set(spec.nodes.map((node) => node.id));
  const structuralEdges = spec.edges.filter((edge) => (edge.weight ?? 1) >= 0.5);
  const parentCounts = new Map(spec.nodes.map((node) => [node.id, 0]));
  for (const edge of structuralEdges) {
    if (!ids.has(edge.source) || !ids.has(edge.target)) continue;
    parentCounts.set(edge.target, (parentCounts.get(edge.target) ?? 0) + 1);
  }
  const roots = [...parentCounts.entries()].filter(([, count]) => count === 0).map(([id]) => id);
  return {
    nodes: spec.nodes.length,
    edges: spec.edges.length,
    structural_edges: structuralEdges.length,
    structural_roots: roots,
    hierarchy_eligible:
      roots.length === 1 &&
      ![...parentCounts.values()].some((count) => count > 1) &&
      !spec.nodes.some((node) => node.id !== roots[0] && node.x != null && node.y != null)
  };
}

function layoutStats(layout, spec) {
  const points = Object.values(layout ?? {});
  if (points.length === 0) return null;
  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  const selected = spec.selected_node_id ? layout?.[spec.selected_node_id] ?? null : null;
  const componentPoints = spec.nodes
    .filter((node) => componentIdFromNodeId(node.id))
    .map((node) => layout?.[node.id])
    .filter(Boolean);
  return {
    width: Math.max(...xs) - Math.min(...xs),
    height: Math.max(...ys) - Math.min(...ys),
    selected_component_avg_distance: selected && componentPoints.length > 0
      ? componentPoints.reduce((sum, point) => sum + Math.hypot(point.x - selected.x, point.y - selected.y), 0) / componentPoints.length
      : null
  };
}

function installDebugApi() {
  window.navilNetworkDebug = {
    state,
    buildSpec,
    getLayout() {
      if (state.sessionHandle == null || typeof wasm.get_network_layout_session !== "function") return null;
      return wasm.get_network_layout_session(state.sessionHandle);
    },
    getStats() {
      const spec = buildSpec();
      const layout = this.getLayout();
      return {
        mode: state.activeSubgraphLayout ?? "whole",
        selected_node_id: state.selectedNodeId,
        spec: structuralStats(spec),
        layout: layoutStats(layout, spec)
      };
    }
  };
}

function syncCanvasSize() {
  const rect = dom.frame.getBoundingClientRect();
  const cssWidth = Math.max(360, Math.round(rect.width));
  const cssHeight = Math.max(360, Math.round(rect.height));
  const width = Math.round(cssWidth * state.pixelRatio);
  const height = Math.round(cssHeight * state.pixelRatio);
  if (dom.canvas.width === width && dom.canvas.height === height) {
    return false;
  }
  dom.canvas.width = width;
  dom.canvas.height = height;
  return true;
}

function eventPoint(event) {
  const rect = dom.canvas.getBoundingClientRect();
  return {
    x: (event.clientX - rect.left) * (dom.canvas.width / rect.width),
    y: (event.clientY - rect.top) * (dom.canvas.height / rect.height)
  };
}

function buildSpec() {
  if (isNavilLayoutSubgraph()) {
    return buildNavilLayoutSubgraphSpec();
  }

  const rows = visibleEntries();
  const visible = rows.map((row) => row.entry);
  const visibleIds = new Set(visible.map((entry) => entry.order_index));
  const nodes = [
    {
      id: ROOT_NODE_ID,
      label: "",
      x: 0,
      y: 0,
      color: "#0f766e",
      style: { radius: 1, opacity: 0, label_visible: false, stroke_width: 0 },
      force_layers: [0],
      properties: {}
    }
  ];
  const edges = [];

  for (const entry of visible) {
    const orderIndex = entry.order_index;
    const id = nodeId(orderIndex);
    const color = nodeColor(entry);
    const childCount = entryChildren(entry).length;
    const expanded = state.expandedIds.has(id);
    const radius = relevanceRadius(entry);
    const stage = state.stageByOrder.get(orderIndex);

    nodes.push({
      id,
      label: truncateLabel(entry.title),
      color,
      shape: isExerciseEntry(entry) ? "diamond" : entry.level === 1 ? "square" : "circle",
      label_inside: false,
      style: {
        radius: scale(radius, 1),
        label_color: "#18202f",
        stroke_width: scale(stage || entry.refinement_count > 0 ? 2 : 1, 0),
        stroke_color: stage ? color : "#ffffff",
        shadow_color: color,
        shadow_blur: scale(stage ? 8 : 3, 0),
        shadow_offset_x: 0,
        shadow_offset_y: 0,
        shadow_opacity: stage ? 0.4 : 0.12
      },
      force_layers: forceLayers(entry),
      properties: childCount > 0
        ? {
            [TOGGLEABLE_PROPERTY_KEY]: "true",
            [EXPANDED_PROPERTY_KEY]: expanded ? "true" : "false"
          }
        : {}
    });

    const parentId =
      entry.parent_order_index == null || !visibleIds.has(entry.parent_order_index)
        ? ROOT_NODE_ID
        : nodeId(entry.parent_order_index);
    const hideRootEdge = parentId === ROOT_NODE_ID;
    edges.push({
      source: parentId,
      target: id,
      weight: expanded ? EXPANDED_PARENT_EDGE_WEIGHT : HIERARCHY_EDGE_WEIGHT,
      style: hideRootEdge
        ? { stroke_width: 0, opacity: 0 }
        : { stroke_color: "#7b8794", stroke_width: scale(1.1, 0), opacity: 0.38 }
    });
  }

  const components = activeComponents();
  const componentsByParent = new Map();
  for (const component of components) {
    if (!visibleIds.has(component.parent_toc_order_index)) continue;
    const parentId = nodeId(component.parent_toc_order_index);
    const parentEntry = state.entriesByOrder.get(component.parent_toc_order_index);
    const parentLayers = parentEntry ? forceLayers(parentEntry) : [component.parent_toc_order_index];
    const bucket = componentsByParent.get(component.parent_toc_order_index) ?? [];
    bucket.push(component);
    componentsByParent.set(component.parent_toc_order_index, bucket);

    nodes.push({
      id: componentNodeId(component.id),
      label: componentLabel(component),
      color: componentColor(component),
      shape: component.role === "question" ? "diamond" : component.role === "definition" ? "square" : "circle",
      label_inside: false,
      style: {
        radius: scale(component.role === "question" ? 12 : 10, 1),
        label_color: "#18202f",
        stroke_width: scale(1.5, 0),
        stroke_color: "#ffffff",
        shadow_color: componentColor(component),
        shadow_blur: scale(4, 0),
        shadow_offset_x: 0,
        shadow_offset_y: 0,
        shadow_opacity: 0.22
      },
      force_layers: [...parentLayers, COMPONENT_FORCE_LAYER_OFFSET + component.parent_toc_order_index],
      properties: {
        role: component.role,
        page: String(component.page_no),
        sequence: String(component.sequence)
      }
    });

    edges.push({
      source: parentId,
      target: componentNodeId(component.id),
      weight: 0.62,
      style: {
        stroke_color: componentColor(component),
        stroke_width: scale(1, 0),
        opacity: 0.34
      }
    });
  }

  for (const group of componentsByParent.values()) {
    group.sort((a, b) => a.page_no - b.page_no || a.sequence - b.sequence || a.id.localeCompare(b.id));
    for (let i = 0; i + 1 < group.length; i += 1) {
      edges.push({
        source: componentNodeId(group[i].id),
        target: componentNodeId(group[i + 1].id),
        weight: SIBLING_EDGE_WEIGHT,
        style: {
          stroke_color: "#9aa5b1",
          stroke_width: scale(0.7, 0),
          dash_pattern: [scale(4, 1), scale(4, 1)],
          opacity: 0.2
        }
      });
    }
  }

  for (const siblings of state.childrenByParent.values()) {
    const shown = siblings.filter((entry) => visibleIds.has(entry.order_index));
    for (let i = 0; i + 1 < shown.length; i += 1) {
      edges.push({
        source: nodeId(shown[i].order_index),
        target: nodeId(shown[i + 1].order_index),
        weight: SIBLING_EDGE_WEIGHT,
        style: {
          stroke_color: "#9aa5b1",
          stroke_width: scale(0.8, 0),
          dash_pattern: [scale(6, 1), scale(4, 1)],
          opacity: 0.18
        }
      });
    }
  }

  return {
    width: dom.canvas.width,
    height: dom.canvas.height,
    pixel_ratio: state.pixelRatio,
    title: state.data.book.title,
    nodes,
    edges,
    show_arrows: false,
    show_labels: true,
    node_radius: scale(16, 1),
    layout_iterations: 260,
    spring_length_scale: 1.05,
    cooling_rate: 0.92,
    selected_node_id: state.selectedNodeId,
    default_edge_style: { stroke_color: "#8b96a5", stroke_width: scale(1, 0), opacity: 0.3 },
    selection_style: { stroke_color: "#0f172a", stroke_width: scale(4, 1), padding: scale(8, 1) },
    margin: scale(48, 1)
  };
}

function buildNavilLayoutSubgraphSpec() {
  const subgraph = activeSubgraph();
  const rootEntry = subgraph ? state.entriesByOrder.get(subgraph.root_order_index) : null;
  const variant = activeNavilVariant();
  if (!subgraph || !rootEntry) {
    throw new Error("Navil layout subgraph fixture is missing.");
  }
  if (!variant) {
    throw new Error("Navil layout variant is missing.");
  }

  const allowedOrderSet = stageFilteredOrderSet(subgraph);
  const rows = visibleEntriesFromTopEntries(topEntriesForOrderSet(allowedOrderSet), allowedOrderSet);
  const visible = rows.map((row) => row.entry);
  const visibleIds = new Set(visible.map((entry) => entry.order_index));
  const nodes = [
    {
      id: ROOT_NODE_ID,
      label: "",
      x: 0,
      y: 0,
      color: "#0f766e",
      style: { radius: 1, opacity: 0, label_visible: false, stroke_width: 0 },
      force_layers: [0],
      properties: {}
    }
  ];
  const edges = [];
  const navilNodeRadius = scale(21, 1);
  const navilComponentRadius = scale(17, 1);
  const demoBaseNodeRadius = scale(16, 1);

  for (const entry of visible) {
    const orderIndex = entry.order_index;
    const id = nodeId(orderIndex);
    const color = nodeColor(entry);
    const childCount = entryChildren(entry).length;
    const expanded = state.expandedIds.has(id);
    const stage = state.stageByOrder.get(orderIndex);
    const radius = variant.useDemoParams ? scale(relevanceRadius(entry), 1) : navilNodeRadius;

    nodes.push({
      id,
      label: variant.useDemoParams ? truncateLabel(entry.title) : entry.title.length > 22 ? `${entry.title.slice(0, 20)}...` : entry.title,
      color,
      shape: variant.useDemoParams
        ? isExerciseEntry(entry) ? "diamond" : entry.level === 1 ? "square" : "circle"
        : undefined,
      label_inside: false,
      style: {
        radius,
        label_color: "#18202f",
        stroke_width: scale(variant.useDemoParams ? stage || entry.refinement_count > 0 ? 2 : 1 : stage ? 2 : 0, 0),
        stroke_color: stage ? color : variant.useDemoParams ? "#ffffff" : undefined,
        shadow_color: stage || variant.useDemoParams ? color : "#000000",
        shadow_blur: scale(stage ? 8 : 3, 0),
        shadow_offset_x: 0,
        shadow_offset_y: 0,
        shadow_opacity: stage ? variant.useDemoParams ? 0.4 : 0.5 : variant.useDemoParams ? 0.12 : 0.1
      },
      force_layers: forceLayers(entry),
      properties: childCount > 0
        ? {
            [TOGGLEABLE_PROPERTY_KEY]: "true",
            [EXPANDED_PROPERTY_KEY]: expanded ? "true" : "false"
          }
        : {}
    });

    const parentId =
      entry.parent_order_index == null || !visibleIds.has(entry.parent_order_index)
        ? ROOT_NODE_ID
        : nodeId(entry.parent_order_index);
    edges.push({
      source: parentId,
      target: id,
      weight: expanded ? EXPANDED_PARENT_EDGE_WEIGHT : HIERARCHY_EDGE_WEIGHT,
      style: parentId === ROOT_NODE_ID ? { stroke_width: 0, opacity: 0 } : undefined
    });
  }

  const componentsByParent = new Map();
  for (const component of activeComponents()) {
    if (!visibleIds.has(component.parent_toc_order_index)) continue;
    const parentId = nodeId(component.parent_toc_order_index);
    const parentEntry = state.entriesByOrder.get(component.parent_toc_order_index);
    const parentLayers = parentEntry ? forceLayers(parentEntry) : [component.parent_toc_order_index];
    const bucket = componentsByParent.get(component.parent_toc_order_index) ?? [];
    bucket.push(component);
    componentsByParent.set(component.parent_toc_order_index, bucket);

    nodes.push({
      id: componentNodeId(component.id),
      label: componentLabel(component),
      color: componentColor(component),
      shape: variant.useDemoParams
        ? component.role === "question" ? "diamond" : component.role === "definition" ? "square" : "circle"
        : undefined,
      label_inside: false,
      style: {
        radius: variant.useDemoParams ? scale(component.role === "question" ? 12 : 10, 1) : navilComponentRadius,
        label_color: variant.useDemoParams ? "#18202f" : "#657186",
        stroke_width: variant.useDemoParams ? scale(1.5, 0) : 0,
        stroke_color: variant.useDemoParams ? "#ffffff" : undefined,
        shadow_color: variant.useDemoParams ? componentColor(component) : "#000000",
        shadow_blur: variant.useDemoParams ? scale(4, 0) : scale(2, 0),
        shadow_offset_x: 0,
        shadow_offset_y: 0,
        shadow_opacity: variant.useDemoParams ? 0.22 : 0.1
      },
      force_layers: [...parentLayers, COMPONENT_FORCE_LAYER_OFFSET + component.parent_toc_order_index],
      properties: {
        navil_subgraph_role: ATTACHED_CUSTOM_SUBGRAPH_ROLE,
        role: component.role,
        page: String(component.page_no),
        sequence: String(component.sequence)
      }
    });

    edges.push({
      source: parentId,
      target: componentNodeId(component.id),
      weight: variant.componentEdgeWeight,
      style: variant.useDemoParams
        ? {
            stroke_color: componentColor(component),
            stroke_width: scale(1, 0),
            opacity: 0.34
          }
        : undefined
    });
  }

  for (const group of componentsByParent.values()) {
    group.sort((a, b) => a.page_no - b.page_no || a.sequence - b.sequence || a.id.localeCompare(b.id));
    for (let i = 0; i + 1 < group.length; i += 1) {
      edges.push({
        source: componentNodeId(group[i].id),
        target: componentNodeId(group[i + 1].id),
        weight: SIBLING_EDGE_WEIGHT,
        style: {
          stroke_color: "#9aa5b1",
          stroke_width: scale(0.8, 0),
          dash_pattern: [scale(6, 1), scale(4, 1)],
          opacity: 0.18
        }
      });
    }
  }

  for (const siblings of state.childrenByParent.values()) {
    const shown = siblings.filter((entry) => visibleIds.has(entry.order_index));
    for (let i = 0; i + 1 < shown.length; i += 1) {
      edges.push({
        source: nodeId(shown[i].order_index),
        target: nodeId(shown[i + 1].order_index),
        weight: SIBLING_EDGE_WEIGHT,
        style: {
          stroke_color: "#9aa5b1",
          stroke_width: scale(0.8, 0),
          dash_pattern: [scale(6, 1), scale(4, 1)],
          opacity: 0.18
        }
      });
    }
  }

  const baseSpec = {
    width: dom.canvas.width,
    height: dom.canvas.height,
    pixel_ratio: state.pixelRatio,
    title: state.data.book.title,
    nodes,
    edges,
    show_arrows: false,
    show_labels: true,
    node_radius: variant.useDemoParams ? demoBaseNodeRadius : navilNodeRadius,
    layout_iterations: variant.useDemoParams ? 260 : 300,
    spring_length_scale: variant.useDemoParams ? 1.05 : 1.0,
    cooling_rate: 0.92,
    selected_node_id: state.selectedNodeId,
    default_edge_style: variant.useDemoParams
      ? { stroke_color: "#8b96a5", stroke_width: scale(1, 0), opacity: 0.3 }
      : { stroke_color: "#657186", stroke_width: scale(1, 0), opacity: 0.32 },
    selection_style: variant.useDemoParams
      ? { stroke_color: "#0f172a", stroke_width: scale(4, 1), padding: scale(8, 1) }
      : { stroke_color: "#9ca3af", stroke_width: scale(4, 1), padding: scale(8, 1) },
    margin: scale(variant.useDemoParams ? 48 : 40, 1)
  };
  return inducedSubgraph(
    baseSpec,
    buildAttachedSubgraphNodeIds(baseSpec, rootEntry, { keepRootAnchor: variant.keepRootAnchor }),
    { rootEntry, keepRootAnchor: variant.keepRootAnchor }
  );
}

function destroySession() {
  cancelTopologyAnimation();
  if (state.sessionHandle == null || typeof wasm.destroy_network_session !== "function") {
    state.sessionHandle = null;
    return;
  }
  try {
    wasm.destroy_network_session(state.sessionHandle);
  } finally {
    state.sessionHandle = null;
  }
}

function cancelTopologyAnimation() {
  if (state.transitionFrame != null) {
    cancelAnimationFrame(state.transitionFrame);
    state.transitionFrame = null;
  }
}

function syncSession({ recreate = false } = {}) {
  const spec = buildSpec();
  if (state.sessionHandle == null || recreate || typeof wasm.update_network_session !== "function") {
    destroySession();
    state.sessionHandle = wasm.create_network_session("network-canvas", spec);
    return;
  }
  wasm.update_network_session(state.sessionHandle, spec);
}

function animateTopologyTransition() {
  if (
    state.sessionHandle == null ||
    typeof wasm.has_network_transition_session !== "function" ||
    typeof wasm.render_network_transition_session !== "function" ||
    !wasm.has_network_transition_session(state.sessionHandle)
  ) {
    wasm.render_network_session(state.sessionHandle);
    return;
  }

  const handle = state.sessionHandle;
  const startedAt = performance.now();

  const tick = (now) => {
    if (state.sessionHandle !== handle) {
      state.transitionFrame = null;
      return;
    }

    const t = Math.min(1, Math.max(0, (now - startedAt) / TOPOLOGY_ANIMATION_MS));
    try {
      wasm.render_network_transition_session(handle, easeInOutCubic(t));
    } catch (error) {
      console.error("[navil-network] transition render failed", error);
      wasm.render_network_session(handle);
      state.transitionFrame = null;
      return;
    }

    if (t < 1) {
      state.transitionFrame = requestAnimationFrame(tick);
      return;
    }

    state.transitionFrame = null;
    if (typeof wasm.clear_network_transition_session === "function") {
      wasm.clear_network_transition_session(handle);
    }
    wasm.render_network_session(handle);
  };

  wasm.render_network_transition_session(handle, 0);
  state.transitionFrame = requestAnimationFrame(tick);
}

function renderGraph({ recreate = false, animateTopology = false } = {}) {
  cancelTopologyAnimation();
  const sizeChanged = syncCanvasSize();
  syncSession({ recreate: recreate || sizeChanged });
  renderMetrics();
  if (animateTopology && !recreate && !sizeChanged) {
    animateTopologyTransition();
  } else {
    wasm.render_network_session(state.sessionHandle);
  }
}

function focusSelectedNode() {
  if (state.sessionHandle == null || !state.selectedNodeId) return;
  if (typeof wasm.compute_network_focus_view_session !== "function") return;
  const view = wasm.compute_network_focus_view_session(state.sessionHandle, state.selectedNodeId, {
    padding: scale(72, 1),
    min_world_span: scale(240, 1)
  });
  if (view && typeof wasm.set_network_view_session === "function") {
    wasm.set_network_view_session(state.sessionHandle, view);
    wasm.render_network_session(state.sessionHandle);
  }
}

function renderMetrics() {
  const visible = visibleEntries().length;
  const componentCount = activeComponents().length;
  const spec = buildSpec();
  dom.visibleCount.textContent = activeSubgraph()
    ? `${visible} TOC + ${componentCount} refined`
    : `${visible} / ${state.data.entries.length}`;
  dom.metricVisible.textContent = activeSubgraph() ? `${visible} + ${componentCount}` : String(visible);
  dom.metricEdges.textContent = String(spec.edges.filter((edge) => edge.style?.opacity !== 0).length);
  const subgraph = activeSubgraph();
  dom.metricPages.textContent = subgraph
    ? `${subgraph.page_start}-${subgraph.page_end}`
    : `${state.data.book.page_start}-${state.data.book.page_end}`;
  dom.metricStages.textContent = String(state.data.path?.path_stages?.length ?? 0);
}

function renderSidebar() {
  dom.loadVectorSubgraph.classList.toggle(
    "is-active",
    state.activeSubgraphId === COMPONENT_SUBGRAPH_ID && state.activeSubgraphLayout === SUBGRAPH_LAYOUT.DEMO
  );
  dom.loadNavilSubgraph.classList.toggle(
    "is-active",
    state.activeSubgraphId === COMPONENT_SUBGRAPH_ID && state.activeSubgraphLayout === SUBGRAPH_LAYOUT.NAVIL
  );
  dom.loadNavilStrongSubgraph.classList.toggle(
    "is-active",
    state.activeSubgraphId === COMPONENT_SUBGRAPH_ID && state.activeSubgraphLayout === SUBGRAPH_LAYOUT.NAVIL_STRONG
  );
  dom.loadNavilAnchorSubgraph.classList.toggle(
    "is-active",
    state.activeSubgraphId === COMPONENT_SUBGRAPH_ID && state.activeSubgraphLayout === SUBGRAPH_LAYOUT.NAVIL_ANCHOR
  );
  dom.loadNavilAnchorStrongSubgraph.classList.toggle(
    "is-active",
    state.activeSubgraphId === COMPONENT_SUBGRAPH_ID && state.activeSubgraphLayout === SUBGRAPH_LAYOUT.NAVIL_ANCHOR_STRONG
  );
  dom.loadNavilDemoParamsSubgraph.classList.toggle(
    "is-active",
    state.activeSubgraphId === COMPONENT_SUBGRAPH_ID && state.activeSubgraphLayout === SUBGRAPH_LAYOUT.NAVIL_DEMO_PARAMS
  );
  dom.wholeGraph.classList.toggle("is-active", state.activeSubgraphId == null);
  const fragment = document.createDocumentFragment();
  for (const { entry, depth } of visibleEntries()) {
    const id = nodeId(entry.order_index);
    const row = document.createElement("button");
    row.type = "button";
    row.className = "toc-row";
    row.dataset.nodeId = id;
    row.classList.toggle("is-selected", state.selectedNodeId === id);
    row.style.paddingLeft = `${8 + depth * 14}px`;
    row.addEventListener("click", () => selectEntry(entry.order_index, { focus: true }));

    if (hasChildren(entry)) {
      const toggle = document.createElement("span");
      toggle.className = "toc-toggle";
      toggle.textContent = state.expandedIds.has(id) ? "-" : "+";
      toggle.addEventListener("click", (event) => {
        event.stopPropagation();
        toggleEntry(entry.order_index);
      });
      row.append(toggle);
    } else {
      const spacer = document.createElement("span");
      spacer.className = "toc-spacer";
      row.append(spacer);
    }

    const stage = document.createElement("span");
    stage.className = "toc-stage";
    stage.style.background = stageColor(entry.order_index) ?? levelColor(entry);
    row.append(stage);

    const title = document.createElement("span");
    title.className = "toc-title";
    title.textContent = entry.title;
    row.append(title);

    const page = document.createElement("span");
    page.className = "toc-page";
    page.textContent = String(entry.page_number);
    row.append(page);

    fragment.append(row);
  }
  dom.tocList.replaceChildren(fragment);
  renderMetrics();
}

function scrollSelectedIntoView() {
  if (!state.selectedNodeId || typeof CSS === "undefined" || typeof CSS.escape !== "function") return;
  const row = dom.tocList.querySelector(`[data-node-id="${CSS.escape(state.selectedNodeId)}"]`);
  row?.scrollIntoView({ block: "nearest" });
}

function detailCard(label, value) {
  const card = document.createElement("div");
  card.className = "details-card";
  const l = document.createElement("span");
  l.className = "details-label";
  l.textContent = label;
  const v = document.createElement("span");
  v.className = "details-value";
  v.textContent = value;
  card.append(l, v);
  return card;
}

function renderComponentDetails(component) {
  const stack = document.createElement("div");
  const title = document.createElement("h2");
  title.textContent = component.description || componentLabel(component);
  const parent = state.entriesByOrder.get(component.parent_toc_order_index);
  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = `${component.role} - page ${component.page_no} - ${parent?.title ?? "local section"}`;

  const grid = document.createElement("div");
  grid.className = "details-grid";
  grid.append(
    detailCard("Role", component.role),
    detailCard("Page", String(component.page_no)),
    detailCard("Sequence", String(component.sequence)),
    detailCard("Parent", parent ? parent.title : String(component.parent_toc_order_index))
  );

  const card = document.createElement("section");
  card.className = "stage-card";
  const label = document.createElement("span");
  label.className = "stage-label";
  label.textContent = "Refined component";
  const copy = document.createElement("p");
  copy.textContent = component.preview || component.text || "No preview exported for this component.";
  card.append(label, copy);

  stack.append(title, meta, grid, card);
  dom.details.replaceChildren(stack);
}

function renderDetails() {
  const componentId = componentIdFromNodeId(state.selectedNodeId);
  if (componentId) {
    const component = findComponent(componentId);
    if (component) {
      renderComponentDetails(component);
      return;
    }
  }

  const orderIndex = orderIndexFromNodeId(state.selectedNodeId);
  const entry = orderIndex == null ? null : state.entriesByOrder.get(orderIndex);
  if (!entry) {
    const empty = document.createElement("p");
    empty.className = "details-empty";
    empty.textContent = "Select a TOC entry to inspect page, hierarchy, path, and copied Navil aggregates.";
    dom.details.replaceChildren(empty);
    return;
  }

  const stack = document.createElement("div");
  const title = document.createElement("h2");
  title.textContent = entry.title;
  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = `Order ${entry.order_index} - level ${entry.level} - page ${entry.page_number}`;

  const grid = document.createElement("div");
  grid.className = "details-grid";
  grid.append(
    detailCard("Children", String(entry.child_count ?? 0)),
    detailCard("Chunks", String(entry.chunk_count ?? 0)),
    detailCard("Refinements", String(entry.refinement_count ?? 0)),
    detailCard("Relevance", formatRelevance(entry)),
    detailCard("Normalized", normalizedRelevance(entry).toFixed(2)),
    detailCard("Path", entry.path ?? "none")
  );

  stack.append(title, meta, grid);

  if (entry.relevance_source === "parent" && entry.relevance_parent_order_index != null) {
    const parent = state.entriesByOrder.get(entry.relevance_parent_order_index);
    const card = document.createElement("section");
    card.className = "stage-card";
    const label = document.createElement("span");
    label.className = "stage-label";
    label.textContent = "Inherited relevance";
    const copy = document.createElement("p");
    copy.textContent = parent
      ? `No direct relevance row; inherited from ${parent.title}.`
      : "No direct relevance row; inherited from parent.";
    card.append(label, copy);
    stack.append(card);
  }

  const ranked = state.rankByOrder.get(entry.order_index);
  const stage = state.stageByOrder.get(entry.order_index);
  if (ranked || stage) {
    const card = document.createElement("section");
    card.className = "stage-card";
    const label = document.createElement("span");
    label.className = "stage-label";
    label.textContent = stage ? `Stage ${stage.stageIndex + 1}` : "Ranked TOC";
    const heading = document.createElement("p");
    heading.textContent = stage?.title ?? "Not assigned to a path stage";
    const copy = document.createElement("p");
    copy.textContent = ranked?.rationale ?? stage?.rationale ?? stage?.focus ?? "No rationale copied for this entry.";
    card.append(label, heading, copy);
    stack.append(card);
  }

  const roles = roleSummary(entry);
  if (roles.length > 0) {
    const card = document.createElement("section");
    card.className = "role-card";
    const label = document.createElement("span");
    label.className = "stage-label";
    label.textContent = "Refinement roles";
    const list = document.createElement("div");
    list.className = "role-list";
    for (const [role, count] of roles) {
      const pill = document.createElement("span");
      pill.className = "role-pill";
      pill.textContent = `${role} ${count}`;
      list.append(pill);
    }
    card.append(label, list);
    stack.append(card);
  }

  dom.details.replaceChildren(stack);
}

function selectEntry(orderIndex, { focus = false, scroll = true } = {}) {
  const entry = state.entriesByOrder.get(orderIndex);
  if (!entry) return;
  expandAncestors(entry);
  state.selectedNodeId = nodeId(orderIndex);
  renderSidebar();
  renderDetails();
  renderGraph({ animateTopology: true });
  if (state.sessionHandle != null && typeof wasm.set_network_selection === "function") {
    wasm.set_network_selection(state.sessionHandle, state.selectedNodeId);
  }
  if (focus) {
    focusSelectedNode();
  }
  if (scroll) {
    scrollSelectedIntoView();
  }
}

function toggleEntry(orderIndex) {
  const entry = state.entriesByOrder.get(orderIndex);
  if (!entry || !hasChildren(entry)) return;
  const id = nodeId(orderIndex);
  if (state.expandedIds.has(id)) {
    state.expandedIds.delete(id);
    const selectedOrder = orderIndexFromNodeId(state.selectedNodeId);
    const selectedEntry = selectedOrder == null ? null : state.entriesByOrder.get(selectedOrder);
    if (selectedEntry && isDescendantOf(selectedEntry, orderIndex)) {
      state.selectedNodeId = id;
    }
  } else {
    state.expandedIds.add(id);
  }
  renderSidebar();
  renderDetails();
  renderGraph({ animateTopology: true });
  scrollSelectedIntoView();
}

function resetDemo() {
  state.expandedIds = initialExpandedIds();
  const subgraph = activeSubgraph();
  const initial = subgraph ? state.entriesByOrder.get(subgraph.root_order_index) : findInitialSelection();
  state.selectedNodeId = initial ? nodeId(initial.order_index) : null;
  if (initial && (!subgraph || activeNavilVariant())) expandAncestors(initial);
  renderSidebar();
  renderDetails();
  renderGraph({ recreate: true });
  focusSelectedNode();
  scrollSelectedIntoView();
}

function expandAll() {
  const subgraph = activeSubgraph();
  const entries = subgraph
    ? subgraph.toc_order_indices.map((orderIndex) => state.entriesByOrder.get(orderIndex)).filter(Boolean)
    : state.data.entries;
  state.expandedIds = new Set(entries.filter((entry) => hasChildren(entry)).map((entry) => nodeId(entry.order_index)));
  renderSidebar();
  renderDetails();
  renderGraph({ animateTopology: true });
  scrollSelectedIntoView();
}

function collapseToChapters() {
  if (activeSubgraph()) {
    loadComponentSubgraph(state.activeSubgraphLayout ?? SUBGRAPH_LAYOUT.DEMO);
    return;
  }
  state.expandedIds = initialExpandedIds();
  const selectedOrder = orderIndexFromNodeId(state.selectedNodeId);
  const selectedEntry = selectedOrder == null ? null : state.entriesByOrder.get(selectedOrder);
  if (selectedEntry && selectedEntry.parent_order_index != null) {
    const topAncestor = ancestorOrderIndices(selectedEntry).at(-1);
    state.selectedNodeId = topAncestor != null ? nodeId(topAncestor) : state.selectedNodeId;
  }
  renderSidebar();
  renderDetails();
  renderGraph({ animateTopology: true });
  scrollSelectedIntoView();
}

function loadComponentSubgraph(layout) {
  const subgraph = state.componentSubgraphsById.get(COMPONENT_SUBGRAPH_ID);
  if (!subgraph) return;
  state.activeSubgraphId = COMPONENT_SUBGRAPH_ID;
  state.activeSubgraphLayout = layout;
  state.expandedIds = initialExpandedIds();
  state.selectedNodeId = nodeId(subgraph.root_order_index);
  const root = state.entriesByOrder.get(subgraph.root_order_index);
  if (NAVIL_VARIANTS[layout] && root) {
    expandAncestors(root);
  }
  renderSidebar();
  renderDetails();
  renderGraph({ animateTopology: true });
  focusSelectedNode();
  scrollSelectedIntoView();
}

function loadVectorSubgraph() {
  loadComponentSubgraph(SUBGRAPH_LAYOUT.DEMO);
}

function loadNavilSubgraph() {
  loadComponentSubgraph(SUBGRAPH_LAYOUT.NAVIL);
}

function loadNavilStrongSubgraph() {
  loadComponentSubgraph(SUBGRAPH_LAYOUT.NAVIL_STRONG);
}

function loadNavilAnchorSubgraph() {
  loadComponentSubgraph(SUBGRAPH_LAYOUT.NAVIL_ANCHOR);
}

function loadNavilAnchorStrongSubgraph() {
  loadComponentSubgraph(SUBGRAPH_LAYOUT.NAVIL_ANCHOR_STRONG);
}

function loadNavilDemoParamsSubgraph() {
  loadComponentSubgraph(SUBGRAPH_LAYOUT.NAVIL_DEMO_PARAMS);
}

function showWholeGraph() {
  state.activeSubgraphId = null;
  state.activeSubgraphLayout = null;
  state.expandedIds = initialExpandedIds();
  const initial = findInitialSelection();
  state.selectedNodeId = initial ? nodeId(initial.order_index) : null;
  if (initial) expandAncestors(initial);
  renderSidebar();
  renderDetails();
  renderGraph({ animateTopology: true });
  focusSelectedNode();
  scrollSelectedIntoView();
}

function attachControls() {
  dom.reset.addEventListener("click", resetDemo);
  dom.expandAll.addEventListener("click", expandAll);
  dom.collapseAll.addEventListener("click", collapseToChapters);
  dom.loadVectorSubgraph.addEventListener("click", loadVectorSubgraph);
  dom.loadNavilSubgraph.addEventListener("click", loadNavilSubgraph);
  dom.loadNavilStrongSubgraph.addEventListener("click", loadNavilStrongSubgraph);
  dom.loadNavilAnchorSubgraph.addEventListener("click", loadNavilAnchorSubgraph);
  dom.loadNavilAnchorStrongSubgraph.addEventListener("click", loadNavilAnchorStrongSubgraph);
  dom.loadNavilDemoParamsSubgraph.addEventListener("click", loadNavilDemoParamsSubgraph);
  dom.wholeGraph.addEventListener("click", showWholeGraph);
}

function attachCanvasInteractions() {
  const drag = {
    active: false,
    moved: false,
    start: { x: 0, y: 0 },
    last: { x: 0, y: 0 }
  };

  dom.canvas.addEventListener("pointerdown", (event) => {
    const point = eventPoint(event);
    drag.active = true;
    drag.moved = false;
    drag.start = point;
    drag.last = point;
    dom.canvas.setPointerCapture(event.pointerId);
    dom.canvas.classList.add("is-dragging");
  });

  dom.canvas.addEventListener("pointermove", (event) => {
    if (!drag.active || state.sessionHandle == null) return;
    const point = eventPoint(event);
    const dx = point.x - drag.last.x;
    const dy = point.y - drag.last.y;
    if (dx === 0 && dy === 0) return;
    drag.last = point;
    drag.moved = drag.moved || Math.hypot(point.x - drag.start.x, point.y - drag.start.y) > 4;
    wasm.pan_network_session(state.sessionHandle, dx, dy);
    wasm.render_network_session(state.sessionHandle);
  });

  function finishPointer(event) {
    if (!drag.active) return;
    const point = eventPoint(event);
    const wasClick = !drag.moved || Math.hypot(point.x - drag.start.x, point.y - drag.start.y) <= 4;
    drag.active = false;
    dom.canvas.classList.remove("is-dragging");
    if (dom.canvas.hasPointerCapture(event.pointerId)) {
      dom.canvas.releasePointerCapture(event.pointerId);
    }
    if (!wasClick || state.sessionHandle == null) return;

    try {
      const hit = typeof wasm.pick_network_hit_session === "function"
        ? wasm.pick_network_hit_session(state.sessionHandle, point.x, point.y)
        : wasm.pick_network_node_session(state.sessionHandle, point.x, point.y);
      const hitNodeId = hit?.node_id ?? null;
      const orderIndex = orderIndexFromNodeId(hitNodeId);
      const componentId = componentIdFromNodeId(hitNodeId);
      if (componentId && findComponent(componentId)) {
        state.selectedNodeId = hitNodeId;
        renderSidebar();
        renderDetails();
        renderGraph();
        if (state.sessionHandle != null && typeof wasm.set_network_selection === "function") {
          wasm.set_network_selection(state.sessionHandle, state.selectedNodeId);
        }
        return;
      }
      if (orderIndex == null) {
        state.selectedNodeId = null;
        renderSidebar();
        renderDetails();
        renderGraph();
        return;
      }
      if (hit?.kind === "toggle") {
        toggleEntry(orderIndex);
      } else {
        selectEntry(orderIndex, { focus: false, scroll: true });
      }
    } catch (error) {
      console.error("[navil-network] pick failed", error);
    }
  }

  dom.canvas.addEventListener("pointerup", finishPointer);
  dom.canvas.addEventListener("pointercancel", finishPointer);
  dom.canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      if (state.sessionHandle == null) return;
      const point = eventPoint(event);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      wasm.zoom_network_session(state.sessionHandle, point.x, point.y, factor);
      wasm.render_network_session(state.sessionHandle);
    },
    { passive: false }
  );
}

async function boot() {
  try {
    const [data] = await Promise.all([
      fetch(DATA_URL).then((response) => {
        if (!response.ok) throw new Error(`Snapshot request failed: ${response.status}`);
        return response.json();
      }),
      init()
    ]);
    prepareData(data);
  } catch (error) {
    console.error("[navil-network] boot failed", error);
    setStatus("error", "Demo failed to load.", error?.message ?? String(error));
    return;
  }

  dom.title.textContent = state.data.book.title;
  attachControls();
  attachCanvasInteractions();
  installDebugApi();
  resetDemo();
  clearStatus();

  const observer = new ResizeObserver(() => {
    if (state.resizeFrame != null) {
      cancelAnimationFrame(state.resizeFrame);
    }
    state.resizeFrame = requestAnimationFrame(() => {
      state.resizeFrame = null;
      renderGraph({ recreate: true });
      focusSelectedNode();
    });
  });
  observer.observe(dom.frame);

  window.addEventListener(
    "beforeunload",
    () => {
      observer.disconnect();
      destroySession();
    },
    { once: true }
  );
}

boot();
