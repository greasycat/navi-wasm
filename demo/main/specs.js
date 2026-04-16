const STRESS_COLORS = ["#0f766e", "#2563eb", "#9333ea", "#db2777", "#f97316", "#16a34a"];
const STRESS_CLUSTERS = [
  "cluster-1",
  "cluster-2",
  "cluster-3",
  "cluster-4",
  "cluster-5",
  "cluster-6"
];

export const initialScatterSpec = {
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

export const initialTreeSpec = {
  width: 720,
  height: 420,
  title: "Stellar lineage preview",
  root_id: "cloud",
  node_radius: 18,
  level_gap: 88,
  sibling_gap: 40,
  margin: 28,
  offset_x: 0,
  offset_y: 0,
  selected_node_id: null,
  collapsed_node_ids: [],
  default_edge_style: {
    stroke_color: "#7b8794",
    stroke_width: 2
  },
  selection_style: {
    stroke_color: "#c2410c",
    stroke_width: 3,
    padding: 8
  },
  nodes: [
    {
      id: "cloud",
      name: "Molecular cloud",
      label: "Cloud",
      color: "#0f766e",
      shape: "diamond",
      label_inside: true,
      style: { radius: 24, stroke_color: "#0b3d3a", stroke_width: 2 },
      properties: { phase: "cold gas", tracer: "CO and dust", note: "root branch" }
    },
    {
      id: "solar",
      name: "Solar-mass branch",
      label: "Solar",
      color: "#d97706",
      shape: "square",
      label_inside: true,
      properties: { mass: "0.8 - 1.2 Msun", outcome: "white dwarf" }
    },
    {
      id: "massive",
      name: "Massive-star branch",
      label: "Massive",
      color: "#b42318",
      shape: "square",
      label_inside: true,
      properties: { mass: "> 8 Msun", outcome: "compact remnant" }
    },
    {
      id: "ttauri",
      name: "T Tauri star",
      label: "T Tauri",
      color: "#f59e0b",
      shape: "circle",
      properties: { phase: "pre-main sequence", tracer: "H alpha" }
    },
    {
      id: "solar-main",
      name: "Sun-like main sequence star",
      label: "Solar analog",
      color: "#f97316",
      shape: "circle",
      properties: { phase: "stable burning", duration: "10 Gyr" }
    },
    {
      id: "white-dwarf",
      name: "White dwarf",
      label: "WD",
      color: "#94a3b8",
      shape: "diamond",
      label_inside: true,
      properties: { phase: "compact remnant", cooling: "long" }
    },
    {
      id: "supergiant",
      name: "Blue supergiant",
      label: "Blue SG",
      color: "#ef4444",
      shape: "circle",
      label_inside: true,
      properties: { phase: "luminous", wind: "fast" }
    },
    {
      id: "collapse",
      name: "Core-collapse supernova",
      label: "Collapse",
      color: "#b42318",
      shape: "diamond",
      label_inside: true,
      style: { radius: 21, stroke_color: "#7f1d1d", stroke_width: 2 },
      properties: { phase: "explosion", signal: "optical and neutrino" }
    },
    {
      id: "neutron-star",
      name: "Neutron star",
      label: "NS",
      color: "#7c3aed",
      shape: "square",
      label_inside: true,
      properties: { phase: "compact remnant", signal: "X-ray and radio" }
    }
  ],
  edges: [
    { source: "cloud", target: "solar", style: { stroke_color: "#d97706" } },
    { source: "cloud", target: "massive", style: { stroke_color: "#b42318" } },
    { source: "solar", target: "ttauri" },
    { source: "ttauri", target: "solar-main" },
    { source: "solar-main", target: "white-dwarf", style: { stroke_color: "#94a3b8" } },
    { source: "massive", target: "supergiant", style: { stroke_color: "#ef4444" } },
    { source: "supergiant", target: "collapse", style: { stroke_color: "#b42318", stroke_width: 3 } },
    { source: "collapse", target: "neutron-star", style: { stroke_color: "#7c3aed" } }
  ]
};

export const initialLineSpec = {
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

export const initialBarSpec = {
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

export const initialHeatmapSpec = {
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

export const initialNetworkSpec = {
  width: 720,
  height: 420,
  title: "Transient alert mesh",
  node_radius: 18,
  margin: 28,
  offset_x: 0,
  offset_y: 0,
  selected_node_id: null,
  layout_iterations: 120,
  show_arrows: true,
  show_labels: true,
  default_node_style: {
    stroke_color: "#e6e9ef",
    stroke_width: 2
  },
  default_edge_style: {
    stroke_color: "#7b8696",
    stroke_width: 2,
    label_visible: true
  },
  selection_style: {
    stroke_color: "#d97706",
    stroke_width: 3,
    padding: 8
  },
  nodes: [
    { id: "ztf", label: "ZTF", color: "#0f766e", x: 72, y: 92, shape: "diamond", label_inside: true, properties: { role: "optical feed", cadence: "minutes" } },
    { id: "rubin", label: "Rubin", color: "#1d4ed8", x: 90, y: 238, shape: "diamond", label_inside: true, properties: { role: "optical feed", cadence: "nightly" } },
    { id: "ligo", label: "LIGO", color: "#b42318", x: 124, y: 344, shape: "diamond", label_inside: true, properties: { role: "GW feed", cadence: "event driven" } },
    { id: "broker", label: "Broker", color: "#111827", x: null, y: null, shape: "square", label_inside: true, style: { radius: 24, stroke_color: "#111827", stroke_width: 2 }, properties: { role: "fan-in", latency: "< 10 s" } },
    { id: "classifier", label: "Classifier", color: "#7c3aed", x: null, y: null, shape: "circle", label_inside: true, properties: { role: "enrichment", output: "scores" } },
    { id: "scheduler", label: "Scheduler", color: "#d97706", x: null, y: null, shape: "square", label_inside: true, style: { radius: 21 }, properties: { role: "dispatch", output: "queues" } },
    { id: "archive", label: "Archive", color: "#475569", x: 520, y: 352, shape: "circle", label_inside: true, properties: { role: "storage", output: "candidate records" } },
    { id: "circulars", label: "Circulars", color: "#a61b3f", x: 640, y: 90, shape: "circle", label_inside: true, properties: { role: "distribution", output: "community notices" } }
  ],
  edges: [
    { source: "ztf", target: "broker", label: "alerts", color: null, weight: null, style: { stroke_color: "#0f766e" } },
    { source: "rubin", target: "broker", label: "detections", color: null, weight: null, style: { stroke_color: "#1d4ed8" } },
    { source: "ligo", target: "broker", label: "GW", color: null, weight: null, style: { stroke_color: "#b42318", stroke_width: 3 } },
    { source: "broker", target: "classifier", label: "triage", color: null, weight: null, style: { stroke_width: 3 } },
    { source: "broker", target: "scheduler", label: "ranked", color: null, weight: null, style: { stroke_width: 4, stroke_color: "#d97706" } },
    { source: "classifier", target: "archive", label: "scores", color: null, weight: null, style: { stroke_color: "#7c3aed" } },
    { source: "scheduler", target: "archive", label: "queues", color: null, weight: null, style: { stroke_color: "#475569" } },
    { source: "archive", target: "circulars", label: "release", color: null, weight: null, style: { stroke_color: "#a61b3f", stroke_width: 3 } }
  ]
};

export const TREE_COLLAPSE_ANIMATION_MS = 220;

function seededRandom(seed) {
  let current = seed >>> 0;

  return () => {
    current = (current * 1664525 + 1013904223) >>> 0;
    return current / 4294967296;
  };
}

export function generateStressScatterSpec(count) {
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

export function compactStressLabel(count) {
  if (count >= 1000000) {
    return `${count / 1000000}M`;
  }

  if (count >= 1000) {
    return `${count / 1000}k`;
  }

  return String(count);
}
