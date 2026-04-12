const statusPanel = document.getElementById("status-panel");
const scatterCanvas = document.getElementById("scatter-canvas");
const treeCanvas = document.getElementById("tree-canvas");
const lineCanvas = document.getElementById("line-canvas");
const barCanvas = document.getElementById("bar-canvas");
const heatmapCanvas = document.getElementById("heatmap-canvas");
const networkCanvas = document.getElementById("network-canvas");
const scatterDetails = document.getElementById("scatter-details");
const treeDetails = document.getElementById("tree-details");
const lineDetails = document.getElementById("line-details");
const barDetails = document.getElementById("bar-details");
const heatmapDetails = document.getElementById("heatmap-details");
const networkDetails = document.getElementById("network-details");
const scatterReset = document.getElementById("scatter-reset");
const scatterStressButtons = Array.from(document.querySelectorAll("[data-stress-count]"));
const scatterMetrics = document.getElementById("scatter-metrics");
const treeReset = document.getElementById("tree-reset");
const lineReset = document.getElementById("line-reset");
const barGroupedButton = document.getElementById("bar-grouped");
const barStackedButton = document.getElementById("bar-stacked");
const barReset = document.getElementById("bar-reset");
const heatmapBwrButton = document.getElementById("heatmap-bwr");
const heatmapViridisButton = document.getElementById("heatmap-viridis");
const heatmapGreensButton = document.getElementById("heatmap-greens");
const networkReset = document.getElementById("network-reset");
const STRESS_COLORS = ["#0f766e", "#2563eb", "#9333ea", "#db2777", "#f97316", "#16a34a"];
const STRESS_CLUSTERS = [
  "cluster-1",
  "cluster-2",
  "cluster-3",
  "cluster-4",
  "cluster-5",
  "cluster-6"
];

const initialScatterSpec = {
  width: 720,
  height: 420,
  title: "Scatter sample",
  x_label: "X",
  y_label: "Y",
  x_range: [-5, 11],
  y_range: [-4, 12],
  selected_point_index: null,
  points: [
    {
      x: -4,
      y: -1,
      name: "North Gate",
      label: "A",
      color: "#ef4444",
      radius: 5,
      properties: { cluster: "alpha", score: "11", owner: "ops" }
    },
    {
      x: -1,
      y: 2,
      name: "Moss Relay",
      label: "B",
      color: "#f97316",
      radius: 5,
      properties: { cluster: "alpha", score: "18", owner: "infra" }
    },
    {
      x: 1,
      y: 4,
      name: "Amber Port",
      label: "C",
      color: "#eab308",
      radius: 5,
      properties: { cluster: "beta", score: "24", owner: "search" }
    },
    {
      x: 3,
      y: 1,
      name: "Lime Switch",
      label: "D",
      color: "#22c55e",
      radius: 5,
      properties: { cluster: "beta", score: "14", owner: "edge" }
    },
    {
      x: 5,
      y: 7,
      name: "Glass Beacon",
      label: "E",
      color: "#06b6d4",
      radius: 5,
      properties: { cluster: "gamma", score: "33", owner: "routing" }
    },
    {
      x: 7,
      y: 3,
      name: "Indigo Link",
      label: "F",
      color: "#3b82f6",
      radius: 5,
      properties: { cluster: "gamma", score: "21", owner: "api" }
    },
    {
      x: 9,
      y: 10,
      name: "Crown Signal",
      label: "G",
      color: "#8b5cf6",
      radius: 5,
      properties: { cluster: "delta", score: "41", owner: "ml" }
    }
  ]
};

const initialTreeSpec = {
  width: 720,
  height: 420,
  title: "Rooted tree sample",
  root_id: "root",
  node_radius: 18,
  level_gap: 88,
  sibling_gap: 40,
  margin: 28,
  offset_x: 0,
  offset_y: 0,
  selected_node_id: null,
  nodes: [
    {
      id: "root",
      name: "Gateway Root",
      label: "root",
      color: "#0f172a",
      shape: "diamond",
      label_inside: true,
      properties: { tier: "core", region: "us-east", load: "62%" }
    },
    {
      id: "left",
      name: "Access West",
      label: "left",
      color: "#2563eb",
      shape: "square",
      label_inside: true,
      properties: { tier: "aggregation", region: "us-west", load: "48%" }
    },
    {
      id: "right",
      name: "Access East",
      label: "right",
      color: "#16a34a",
      shape: "square",
      label_inside: true,
      properties: { tier: "aggregation", region: "us-east", load: "51%" }
    },
    {
      id: "left-a",
      name: "Leaf Pine",
      label: "left-a",
      color: "#f97316",
      shape: "circle",
      label_inside: false,
      properties: { tier: "edge", region: "oregon", load: "37%" }
    },
    {
      id: "left-b",
      name: "Leaf Cedar",
      label: "left-b",
      color: "#eab308",
      shape: "triangle",
      label_inside: false,
      properties: { tier: "edge", region: "california", load: "43%" }
    },
    {
      id: "right-a",
      name: "Leaf Birch",
      label: "right-a",
      color: "#7c3aed",
      shape: "circle",
      label_inside: false,
      properties: { tier: "edge", region: "virginia", load: "39%" }
    },
    {
      id: "right-b",
      name: "Leaf Elm",
      label: "right-b",
      color: "#db2777",
      shape: "triangle",
      label_inside: false,
      properties: { tier: "edge", region: "new-york", load: "35%" }
    }
  ],
  edges: [
    { source: "root", target: "left" },
    { source: "root", target: "right" },
    { source: "left", target: "left-a" },
    { source: "left", target: "left-b" },
    { source: "right", target: "right-a" },
    { source: "right", target: "right-b" }
  ]
};

const initialLineSpec = {
  width: 720,
  height: 420,
  title: "Server latency (ms)",
  x_label: "Hour",
  y_label: "Latency (ms)",
  x_range: null,
  y_range: null,
  selected_point: null,
  show_points: true,
  show_legend: true,
  series: [
    {
      label: "API gateway",
      color: "#2563eb",
      stroke_width: 2,
      points: [
        { x: 0, y: 42, label: "00:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 1, y: 38, label: "01:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 2, y: 35, label: "02:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 3, y: 33, label: "03:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 4, y: 36, label: "04:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 5, y: 41, label: "05:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 6, y: 55, label: "06:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 7, y: 78, label: "07:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 8, y: 94, label: "08:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 9, y: 88, label: "09:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 10, y: 82, label: "10:00", properties: { host: "gw-1", region: "us-east" } },
        { x: 11, y: 76, label: "11:00", properties: { host: "gw-1", region: "us-east" } }
      ]
    },
    {
      label: "Auth service",
      color: "#16a34a",
      stroke_width: 2,
      points: [
        { x: 0, y: 18, label: "00:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 1, y: 16, label: "01:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 2, y: 15, label: "02:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 3, y: 14, label: "03:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 4, y: 15, label: "04:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 5, y: 19, label: "05:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 6, y: 28, label: "06:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 7, y: 44, label: "07:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 8, y: 52, label: "08:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 9, y: 49, label: "09:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 10, y: 43, label: "10:00", properties: { host: "auth-2", region: "us-east" } },
        { x: 11, y: 38, label: "11:00", properties: { host: "auth-2", region: "us-east" } }
      ]
    },
    {
      label: "DB proxy",
      color: "#dc2626",
      stroke_width: 2,
      points: [
        { x: 0, y: 62, label: "00:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 1, y: 58, label: "01:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 2, y: 54, label: "02:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 3, y: 51, label: "03:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 4, y: 53, label: "04:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 5, y: 60, label: "05:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 6, y: 85, label: "06:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 7, y: 118, label: "07:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 8, y: 136, label: "08:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 9, y: 128, label: "09:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 10, y: 112, label: "10:00", properties: { host: "db-proxy-3", region: "us-east" } },
        { x: 11, y: 98, label: "11:00", properties: { host: "db-proxy-3", region: "us-east" } }
      ]
    }
  ]
};

const initialBarSpec = {
  width: 720,
  height: 420,
  title: "Quarterly revenue by region",
  x_label: "Quarter",
  y_label: "Revenue ($M)",
  y_max: null,
  categories: ["Q1", "Q2", "Q3", "Q4"],
  variant: "grouped",
  show_legend: true,
  margin: 32,
  selected_bar: null,
  series: [
    {
      label: "North America",
      color: "#2563eb",
      values: [42.1, 55.3, 61.7, 70.2]
    },
    {
      label: "Europe",
      color: "#16a34a",
      values: [28.4, 34.1, 38.9, 44.6]
    },
    {
      label: "APAC",
      color: "#f97316",
      values: [19.7, 25.8, 31.2, 38.5]
    }
  ]
};

const initialHeatmapSpec = {
  width: 720,
  height: 420,
  title: "Service dependency matrix (avg latency ms)",
  row_labels: ["API GW", "Auth", "Search", "DB Proxy", "Cache"],
  col_labels: ["API GW", "Auth", "Search", "DB Proxy", "Cache"],
  cells: [
    [0.0, 12.4, 8.1, 31.5, 4.2],
    [11.8, 0.0, 5.3, 22.1, 2.9],
    [9.2, 6.7, 0.0, 44.8, 3.1],
    [28.6, 19.3, 41.2, 0.0, 1.8],
    [3.9, 2.4, 2.8, 1.6, 0.0]
  ],
  value_range: null,
  palette: "blue_white_red",
  show_values: true,
  margin: 32,
  selected_cell: null
};

const initialNetworkSpec = {
  width: 720,
  height: 420,
  title: "Service call graph",
  node_radius: 20,
  margin: 28,
  offset_x: 0,
  offset_y: 0,
  selected_node_id: null,
  layout_iterations: 120,
  show_arrows: true,
  show_labels: true,
  nodes: [
    { id: "client", label: "Client", color: "#0f172a", x: null, y: null, shape: "diamond", label_inside: true, properties: { type: "external", rps: "2400" } },
    { id: "lb", label: "LB", color: "#2563eb", x: null, y: null, shape: "circle", label_inside: true, properties: { type: "load-balancer", algo: "round-robin" } },
    { id: "api", label: "API GW", color: "#7c3aed", x: null, y: null, shape: "square", label_inside: true, properties: { type: "gateway", version: "v2.4" } },
    { id: "auth", label: "Auth", color: "#16a34a", x: null, y: null, shape: "circle", label_inside: true, properties: { type: "service", version: "v1.9" } },
    { id: "search", label: "Search", color: "#f97316", x: null, y: null, shape: "circle", label_inside: true, properties: { type: "service", version: "v3.1" } },
    { id: "db", label: "DB", color: "#dc2626", x: null, y: null, shape: "triangle", label_inside: true, properties: { type: "database", engine: "postgres" } },
    { id: "cache", label: "Cache", color: "#0891b2", x: null, y: null, shape: "square", label_inside: true, properties: { type: "cache", engine: "redis" } },
    { id: "queue", label: "Queue", color: "#db2777", x: null, y: null, shape: "diamond", label_inside: true, properties: { type: "message-queue", engine: "kafka" } }
  ],
  edges: [
    { source: "client", target: "lb", label: null, color: null, weight: null },
    { source: "lb", target: "api", label: null, color: null, weight: null },
    { source: "api", target: "auth", label: null, color: null, weight: null },
    { source: "api", target: "search", label: null, color: null, weight: null },
    { source: "api", target: "cache", label: null, color: null, weight: null },
    { source: "search", target: "db", label: null, color: null, weight: null },
    { source: "search", target: "cache", label: null, color: null, weight: null },
    { source: "auth", target: "db", label: null, color: null, weight: null },
    { source: "api", target: "queue", label: null, color: null, weight: null }
  ]
};

const state = {
  scatterSpec: cloneSpec(initialScatterSpec),
  treeSpec: cloneSpec(initialTreeSpec),
  lineSpec: cloneSpec(initialLineSpec),
  barSpec: cloneSpec(initialBarSpec),
  heatmapSpec: cloneSpec(initialHeatmapSpec),
  networkSpec: cloneSpec(initialNetworkSpec),
  scatterSessionHandle: null,
  scatterRenderFrame: null,
  lineSessionHandle: null,
  networkSessionHandle: null,
  scatterPerf: {
    pointCount: initialScatterSpec.points.length,
    renderMs: 0,
    mode: "sample"
  }
};

function setStatus(title, body, kind = "info") {
  statusPanel.className = `status-panel status-${kind}`;
  statusPanel.innerHTML = `
    <p class="status-title">${title}</p>
    <p class="status-body">${body}</p>
  `;
}

async function loadWasm() {
  const moduleUrl = new URL("../pkg/navi_plot_wasm.js", import.meta.url);
  const wasm = await import(moduleUrl.href);
  await wasm.default();
  return wasm;
}

function cloneSpec(value) {
  if (globalThis.structuredClone) {
    return globalThis.structuredClone(value);
  }

  return JSON.parse(JSON.stringify(value));
}

function seededRandom(seed) {
  let current = seed >>> 0;

  return () => {
    current = (current * 1664525 + 1013904223) >>> 0;
    return current / 4294967296;
  };
}

function generateStressScatterSpec(count) {
  const random = seededRandom(42);
  const radius = count >= 100000 ? 1 : 2;
  const points = new Array(count);

  for (let index = 0; index < count; index += 1) {
    const cluster = index % STRESS_COLORS.length;
    const angle = index * 0.173;
    const spiral = 12 + cluster * 6 + (index % 97) * 0.08;
    const jitterX = (random() - 0.5) * 5.5;
    const jitterY = (random() - 0.5) * 5.5;
    const x = Math.cos(angle) * spiral + cluster * 18 + jitterX;
    const y = Math.sin(angle) * (spiral * 0.78) + (cluster - 2.5) * 11 + jitterY;

    points[index] = {
      x,
      y,
      name: `P${index + 1}`,
      label: null,
      color: STRESS_COLORS[cluster],
      radius,
      properties: {
        cluster: STRESS_CLUSTERS[cluster],
        band: String(index % 128)
      }
    };
  }

  return {
    width: 720,
    height: 420,
    title: `Scatter stress test (${count.toLocaleString()} points)`,
    x_label: "Synthetic X",
    y_label: "Synthetic Y",
    x_range: null,
    y_range: null,
    selected_point_index: null,
    points
  };
}

function compactStressLabel(count) {
  if (count >= 1000000) {
    return `${count / 1000000}M`;
  }

  if (count >= 1000) {
    return `${count / 1000}k`;
  }

  return String(count);
}

function setScatterControlsDisabled(disabled) {
  scatterReset.disabled = disabled;
  for (const button of scatterStressButtons) {
    button.disabled = disabled;
  }
}

function setCanvasDragging(canvas, dragging) {
  canvas.classList.toggle("is-dragging", dragging);
}

function eventPoint(event, canvas) {
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / rect.width;
  const scaleY = canvas.height / rect.height;
  return {
    x: (event.clientX - rect.left) * scaleX,
    y: (event.clientY - rect.top) * scaleY
  };
}

function selectionName(item, fallback) {
  return item?.name ?? item?.label ?? fallback;
}

function propertyEntries(properties) {
  if (!properties) {
    return [];
  }

  if (properties instanceof Map) {
    return Array.from(properties.entries());
  }

  return Object.entries(properties);
}

function supportsScatterSessions(wasm) {
  return typeof wasm.create_scatter_session === "function";
}

function destroyScatterSession(wasm) {
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

function syncScatterSession(wasm) {
  if (!supportsScatterSessions(wasm)) {
    state.scatterSessionHandle = null;
    return;
  }

  destroyScatterSession(wasm);
  state.scatterSessionHandle = wasm.create_scatter_session("scatter-canvas", state.scatterSpec);
}

function renderScatterOnly(wasm) {
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

function renderTreeOnly(wasm) {
  wasm.render_tree("tree-canvas", state.treeSpec);
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

function renderProperties(properties) {
  const entries = propertyEntries(properties);
  if (!entries.length) {
    return `<p class="details-empty-copy">No custom properties on this selection.</p>`;
  }

  return `
    <dl class="property-list">
      ${entries
        .map(
          ([key, value]) => `
            <div class="property-row">
              <dt>${key}</dt>
              <dd>${value}</dd>
            </div>
          `
        )
        .join("")}
    </dl>
  `;
}

function buildPropertyList(properties) {
  const entries = propertyEntries(properties);
  if (!entries.length) {
    const p = document.createElement("p");
    p.className = "details-empty-copy";
    p.textContent = "No custom properties on this selection.";
    return p;
  }
  const dl = document.createElement("dl");
  dl.className = "property-list";
  for (const [key, value] of entries) {
    const row = document.createElement("div");
    row.className = "property-row";
    const dt = document.createElement("dt");
    dt.textContent = key;
    const dd = document.createElement("dd");
    dd.textContent = value;
    row.append(dt, dd);
    dl.append(row);
  }
  return dl;
}

function renderScatterDetails() {
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

function renderScatterMetrics() {
  scatterMetrics.textContent =
    `${state.scatterPerf.mode} dataset · ${state.scatterPerf.pointCount.toLocaleString()} points · last scatter render ${state.scatterPerf.renderMs.toFixed(1)} ms`;
}

function renderTreeDetails() {
  const nodeId = state.treeSpec.selected_node_id;
  const node = state.treeSpec.nodes.find((entry) => entry.id === nodeId);

  if (!node) {
    treeDetails.innerHTML = `
      <h3>Node details</h3>
      <p class="details-empty-copy">Click a tree node to inspect its name, hierarchy role, and properties.</p>
    `;
    return;
  }

  const childCount = state.treeSpec.edges.filter((edge) => edge.source === node.id).length;
  treeDetails.innerHTML = `
    <h3>${selectionName(node, node.id)}</h3>
    <p class="details-meta">ID ${node.id} · label ${node.label} · children ${childCount}</p>
    ${renderProperties({
      ...node.properties,
      color: node.color ?? "default"
    })}
  `;
}

function setScatterSelection(wasm, index) {
  state.scatterSpec.selected_point_index = index;
  if (supportsScatterSessions(wasm) && state.scatterSessionHandle != null) {
    wasm.set_scatter_selection(state.scatterSessionHandle, index ?? undefined);
  }
}

function setTreeSelection(nodeId) {
  state.treeSpec.selected_node_id = nodeId;
}

// ── Line chart ────────────────────────────────────────────────────────────────

function supportsLineSessions(wasm) {
  return typeof wasm.create_line_session === "function";
}

function destroyLineSession(wasm) {
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

function syncLineSession(wasm) {
  if (!supportsLineSessions(wasm)) {
    state.lineSessionHandle = null;
    return;
  }
  destroyLineSession(wasm);
  state.lineSessionHandle = wasm.create_line_session("line-canvas", state.lineSpec);
}

function renderLineOnly(wasm) {
  if (supportsLineSessions(wasm)) {
    if (state.lineSessionHandle == null) {
      syncLineSession(wasm);
    }
    wasm.render_line_session(state.lineSessionHandle);
  } else {
    wasm.render_line("line-canvas", state.lineSpec);
  }
}

function renderLineDetails() {
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

function attachLineInteractions(wasm) {
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

  lineReset.addEventListener("click", () => {
    state.lineSpec = cloneSpec(initialLineSpec);
    syncLineSession(wasm);
    renderLineOnly(wasm);
    renderLineDetails();
  });
}

// ── Bar chart ─────────────────────────────────────────────────────────────────

function renderBarOnly(wasm) {
  wasm.render_bar("bar-canvas", state.barSpec);
}

function renderBarDetails() {
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

function attachBarInteractions(wasm) {
  barCanvas.addEventListener("click", (event) => {
    const point = eventPoint(event, barCanvas);
    const hit = wasm.pick_bar(state.barSpec, point.x, point.y);
    state.barSpec.selected_bar = hit ? [hit.series_index, hit.category_index] : null;
    renderBarOnly(wasm);
    renderBarDetails();
  });

  barGroupedButton.addEventListener("click", () => {
    state.barSpec.variant = "grouped";
    renderBarOnly(wasm);
  });

  barStackedButton.addEventListener("click", () => {
    state.barSpec.variant = "stacked";
    renderBarOnly(wasm);
  });

  barReset.addEventListener("click", () => {
    state.barSpec = cloneSpec(initialBarSpec);
    renderBarOnly(wasm);
    renderBarDetails();
  });
}

// ── Heatmap ───────────────────────────────────────────────────────────────────

function renderHeatmapOnly(wasm) {
  wasm.render_heatmap("heatmap-canvas", state.heatmapSpec);
}

function renderHeatmapDetails() {
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

function attachHeatmapInteractions(wasm) {
  heatmapCanvas.addEventListener("click", (event) => {
    const point = eventPoint(event, heatmapCanvas);
    const hit = wasm.pick_heatmap_cell(state.heatmapSpec, point.x, point.y);
    state.heatmapSpec.selected_cell = hit ? [hit.row, hit.col] : null;
    renderHeatmapOnly(wasm);
    renderHeatmapDetails();
  });

  heatmapBwrButton.addEventListener("click", () => {
    state.heatmapSpec.palette = "blue_white_red";
    renderHeatmapOnly(wasm);
  });

  heatmapViridisButton.addEventListener("click", () => {
    state.heatmapSpec.palette = "viridis";
    renderHeatmapOnly(wasm);
  });

  heatmapGreensButton.addEventListener("click", () => {
    state.heatmapSpec.palette = "greens";
    renderHeatmapOnly(wasm);
  });
}

// ── Network / DAG ─────────────────────────────────────────────────────────────

function supportsNetworkSessions(wasm) {
  return typeof wasm.create_network_session === "function";
}

function destroyNetworkSession(wasm) {
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

function syncNetworkSession(wasm) {
  if (!supportsNetworkSessions(wasm)) {
    state.networkSessionHandle = null;
    return;
  }
  destroyNetworkSession(wasm);
  state.networkSessionHandle = wasm.create_network_session("network-canvas", state.networkSpec);
}

function renderNetworkOnly(wasm) {
  if (supportsNetworkSessions(wasm)) {
    if (state.networkSessionHandle == null) {
      syncNetworkSession(wasm);
    }
    wasm.render_network_session(state.networkSessionHandle);
  } else {
    wasm.render_network("network-canvas", state.networkSpec);
  }
}

function renderNetworkDetails() {
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

function attachNetworkInteractions(wasm) {
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

  networkReset.addEventListener("click", () => {
    state.networkSpec = cloneSpec(initialNetworkSpec);
    syncNetworkSession(wasm);
    renderNetworkOnly(wasm);
    renderNetworkDetails();
  });
}

// ── Scene init ────────────────────────────────────────────────────────────────

function renderInitialScene(wasm) {
  syncScatterSession(wasm);
  renderScatterOnly(wasm);
  renderTreeOnly(wasm);
  syncLineSession(wasm);
  renderLineOnly(wasm);
  renderBarOnly(wasm);
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

function attachScatterInteractions(wasm) {
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

function attachTreeInteractions(wasm) {
  const dragState = {
    active: false,
    moved: false,
    start: null,
    last: null
  };

  treeCanvas.addEventListener("pointerdown", (event) => {
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
    state.treeSpec = wasm.pan_tree(state.treeSpec, dx, dy);
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
      const hit = wasm.pick_tree_node(state.treeSpec, point.x, point.y);
      setTreeSelection(hit?.node_id ?? null);
      renderTreeOnly(wasm);
      renderTreeDetails();
    }
  }

  treeCanvas.addEventListener("pointerup", finishPointer);
  treeCanvas.addEventListener("pointercancel", finishPointer);

  treeReset.addEventListener("click", () => {
    state.treeSpec = cloneSpec(initialTreeSpec);
    renderTreeOnly(wasm);
    renderTreeDetails();
  });
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
    destroyLineSession(wasm);
    destroyNetworkSession(wasm);
  }, { once: true });
}

(async () => {
  try {
    const wasm = await loadWasm();
    bootInteractiveDemo(wasm);
  } catch (error) {
    console.error(error);
    setStatus(
      "WASM package not found",
      "Build the package from the repo root with <code>wasm-pack build crates/navi_plot_wasm --target web --out-dir ../../pkg</code>, then serve the repository root and open <code>/demo/</code>.",
      "error"
    );
  }
})();
