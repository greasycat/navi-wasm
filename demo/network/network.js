import init, * as wasm from "../../pkg/navi_plot_wasm.js";

const DEMO_IMAGE_SOURCES = {
  "radio-array": new URL("../assets/radio-array.svg", import.meta.url).href,
  "transit-curve": new URL("../assets/transit-curve.svg", import.meta.url).href
};

async function preloadGraphImages() {
  if (typeof wasm.register_graph_image !== "function") {
    return;
  }

  await Promise.all(
    Object.entries(DEMO_IMAGE_SOURCES).map(async ([key, src]) => {
      if (typeof wasm.has_graph_image === "function" && wasm.has_graph_image(key)) {
        return;
      }

      try {
        await wasm.register_graph_image(key, src);
      } catch (error) {
        console.warn(`[network] image preload failed for ${key}`, error);
      }
    })
  );
}

const SCENARIOS = {
  transient: {
    kicker: "Pinned ingest streams feeding a brokered follow-up mesh",
    title: "Transient alert mesh",
    summary:
      "A compact alert-routing network where survey streams stay pinned at the perimeter while the broker, classifier, and scheduler settle into a central coordination core.",
    note:
      "This is the best scenario for showing how edge labels, arrowheads, and per-link emphasis work together. The pinned anchors keep the outer ring stable while the inner graph reorganizes.",
    facts: [
      { label: "Best for", value: "Alert routing, handoff visibility, and edge labels." },
      { label: "Primary signals", value: "Optical alerts, gamma bursts, and GW packets." },
      { label: "Curated lens", value: "A simplified version of real broker-centered follow-up." }
    ],
    legend: [
      { label: "Survey and mission feeds", color: "#0f766e" },
      { label: "Decision core", color: "#111827" },
      { label: "Follow-up instruments", color: "#f59e0b" },
      { label: "Distribution and archive", color: "#a61b3f" }
    ],
    defaults: {
      nodeRadius: 18,
      edgeWidth: 2,
      layoutIterations: 140,
      showLabels: true,
      showArrows: true,
      forceLabelInside: false
    },
    nodes: [
      {
        id: "ztf",
        label: "ZTF",
        name: "Zwicky Transient Facility",
        color: "#0f766e",
        shape: "diamond",
        labelInside: true,
        anchor: [0.08, 0.22],
        properties: { role: "optical feed", cadence: "minutes", interface: "alert stream" }
      },
      {
        id: "rubin",
        label: "Rubin",
        name: "Rubin wide-fast-deep feed",
        color: "#1d4ed8",
        shape: "diamond",
        labelInside: true,
        anchor: [0.12, 0.48],
        properties: { role: "optical feed", cadence: "nightly", interface: "alert stream" }
      },
      {
        id: "fermi",
        label: "Fermi",
        name: "Fermi trigger stream",
        color: "#2563eb",
        shape: "diamond",
        labelInside: true,
        anchor: [0.24, 0.12],
        properties: { role: "gamma-ray feed", cadence: "burst driven", interface: "GCN notices" }
      },
      {
        id: "ligo",
        label: "LIGO",
        name: "GW localization packets",
        color: "#b42318",
        shape: "diamond",
        labelInside: true,
        anchor: [0.14, 0.78],
        properties: { role: "GW feed", cadence: "event driven", interface: "sky maps" }
      },
      {
        id: "broker",
        label: "Broker",
        name: "Alert broker",
        color: "#111827",
        shape: "square",
        labelInside: true,
        style: { radius: 24, stroke_color: "#111827", stroke_width: 2 },
        media: { kind: "icon", icon: "broker", scale: 0.74, tint_color: "#f8fafc" },
        properties: { role: "fan-in", latency: "< 10 s", output: "ranked candidates" }
      },
      {
        id: "classifier",
        label: "Classifier",
        name: "Real-time classifier",
        color: "#7c3aed",
        shape: "circle",
        labelInside: true,
        style: { radius: 21 },
        properties: { role: "enrichment", inputs: "context features", output: "scores" }
      },
      {
        id: "scheduler",
        label: "Scheduler",
        name: "Follow-up scheduler",
        color: "#d97706",
        shape: "square",
        labelInside: true,
        style: { radius: 22 },
        media: { kind: "icon", icon: "camera", scale: 0.68, tint_color: "#fff7ed" },
        properties: { role: "dispatch", horizon: "hours", output: "instrument queues" }
      },
      {
        id: "spectroscopy",
        label: "Spectra",
        name: "Rapid spectroscopy queue",
        color: "#f59e0b",
        shape: "triangle",
        labelInside: true,
        anchor: [0.82, 0.28],
        media: { kind: "icon", icon: "spectrograph", scale: 0.72, tint_color: "#fff7ed" },
        properties: { role: "follow-up", mode: "spectroscopy", latency: "same night" }
      },
      {
        id: "photometry",
        label: "Photometry",
        name: "Fast photometry network",
        color: "#fb923c",
        shape: "triangle",
        anchor: [0.84, 0.53],
        properties: { role: "follow-up", mode: "multi-band photometry", latency: "hours" }
      },
      {
        id: "radio",
        label: "Radio",
        name: "Triggered radio follow-up",
        color: "#14b8a6",
        shape: "triangle",
        labelInside: true,
        anchor: [0.78, 0.79],
        properties: { role: "follow-up", mode: "radio continuum", latency: "days" }
      },
      {
        id: "archive",
        label: "Archive",
        name: "Candidate archive",
        color: "#475569",
        shape: "circle",
        labelInside: true,
        anchor: [0.62, 0.9],
        media: { kind: "icon", icon: "archive", scale: 0.7, tint_color: "#f8fafc" },
        properties: { role: "storage", mode: "query and replay", retention: "full packets" }
      },
      {
        id: "circulars",
        label: "Circulars",
        name: "Community circulars",
        color: "#a61b3f",
        shape: "circle",
        labelInside: true,
        anchor: [0.9, 0.14],
        properties: { role: "distribution", mode: "human-readable notices", latency: "minutes" }
      }
    ],
    edges: [
      { source: "ztf", target: "broker", label: "alerts", style: { stroke_color: "#0f766e" } },
      { source: "rubin", target: "broker", label: "detections", style: { stroke_color: "#1d4ed8" } },
      { source: "fermi", target: "broker", label: "gamma", style: { stroke_color: "#2563eb" } },
      { source: "ligo", target: "broker", label: "GW", style: { stroke_color: "#b42318", stroke_width: 3 } },
      { source: "broker", target: "classifier", label: "triage", style: { stroke_width: 3 } },
      { source: "broker", target: "scheduler", label: "ranked", style: { stroke_width: 4, stroke_color: "#d97706" } },
      { source: "broker", target: "archive", label: "raw packets", style: { stroke_color: "#475569" } },
      { source: "classifier", target: "scheduler", label: "priority", style: { stroke_color: "#7c3aed" } },
      { source: "classifier", target: "archive", label: "scores", style: { stroke_color: "#7c3aed" } },
      { source: "scheduler", target: "spectroscopy", label: "queue", style: { stroke_color: "#f59e0b" } },
      { source: "scheduler", target: "photometry", label: "queue", style: { stroke_color: "#fb923c" } },
      { source: "scheduler", target: "radio", label: "queue", style: { stroke_color: "#14b8a6" } },
      { source: "spectroscopy", target: "archive", label: "reduced", style: { stroke_color: "#f59e0b" } },
      { source: "photometry", target: "archive", label: "light curves", style: { stroke_color: "#fb923c" } },
      { source: "radio", target: "archive", label: "visibilities", style: { stroke_color: "#14b8a6" } },
      { source: "archive", target: "circulars", label: "release", style: { stroke_color: "#a61b3f", stroke_width: 3 } }
    ]
  },
  radio: {
    kicker: "A calibration loop with one intentional cycle",
    title: "Radio array calibration flow",
    summary:
      "A directional calibration graph for a synthetic interferometer. Anchored dishes and timing references feed a central correlator, then iterate through calibration and imaging before landing in the archive.",
    note:
      "This is the clearest example of cycle handling. The self-calibration loop feeds the imager again, so you can see how the graph renderer behaves when the topology is not a pure DAG.",
    facts: [
      { label: "Best for", value: "Cycles, pinned anchors, and bold processing stages." },
      { label: "Primary signals", value: "Visibilities, gain solutions, dirty images." },
      { label: "Curated lens", value: "Inspired by common radio pipeline stages." }
    ],
    legend: [
      { label: "Instrument feeds", color: "#0f766e" },
      { label: "Core processing", color: "#111827" },
      { label: "Calibration branches", color: "#2563eb" },
      { label: "Data products", color: "#d97706" }
    ],
    defaults: {
      nodeRadius: 17,
      edgeWidth: 2,
      layoutIterations: 150,
      showLabels: true,
      showArrows: true,
      forceLabelInside: false
    },
    nodes: [
      {
        id: "dish-core",
        label: "Core dishes",
        name: "Core dish cluster",
        color: "#0f766e",
        shape: "diamond",
        anchor: [0.08, 0.24],
        media: {
          kind: "image",
          image_key: "radio-array",
          fit: "cover",
          scale: 0.82,
          fallback_icon: "dish"
        },
        properties: { role: "feed", baselines: "short", data: "voltages" }
      },
      {
        id: "dish-south",
        label: "South arm",
        name: "South arm dishes",
        color: "#14b8a6",
        shape: "diamond",
        anchor: [0.08, 0.54],
        properties: { role: "feed", baselines: "medium", data: "voltages" }
      },
      {
        id: "dish-east",
        label: "East arm",
        name: "East arm dishes",
        color: "#2dd4bf",
        shape: "diamond",
        anchor: [0.18, 0.82],
        properties: { role: "feed", baselines: "long", data: "voltages" }
      },
      {
        id: "time-ref",
        label: "Time ref",
        name: "Timing reference",
        color: "#2563eb",
        shape: "square",
        labelInside: true,
        anchor: [0.28, 0.12],
        properties: { role: "reference", sync: "maser", data: "clock packets" }
      },
      {
        id: "correlator",
        label: "Correlator",
        name: "FX correlator",
        color: "#111827",
        shape: "square",
        labelInside: true,
        style: { radius: 24, stroke_color: "#111827", stroke_width: 2 },
        media: { kind: "icon", icon: "database", scale: 0.7, tint_color: "#f8fafc" },
        properties: { role: "core", output: "visibilities", cadence: "real time" }
      },
      {
        id: "rfi",
        label: "RFI filter",
        name: "RFI excision stage",
        color: "#0f766e",
        shape: "circle",
        labelInside: true,
        properties: { role: "cleaning", output: "flagged visibilities", cadence: "seconds" }
      },
      {
        id: "gain",
        label: "Gain",
        name: "Gain calibration",
        color: "#2563eb",
        shape: "circle",
        labelInside: true,
        properties: { role: "calibration", output: "gain tables", cadence: "minutes" }
      },
      {
        id: "bandpass",
        label: "Bandpass",
        name: "Bandpass calibration",
        color: "#60a5fa",
        shape: "circle",
        labelInside: true,
        properties: { role: "calibration", output: "bandpass model", cadence: "minutes" }
      },
      {
        id: "imager",
        label: "Imager",
        name: "Wide-field imager",
        color: "#d97706",
        shape: "square",
        labelInside: true,
        style: { radius: 22 },
        properties: { role: "product stage", output: "dirty image", cadence: "minutes" }
      },
      {
        id: "selfcal",
        label: "Self-cal",
        name: "Self-calibration loop",
        color: "#f59e0b",
        shape: "diamond",
        labelInside: true,
        properties: { role: "feedback", output: "refined gains", cadence: "iterative" }
      },
      {
        id: "sourcefinder",
        label: "Source finder",
        name: "Source finder",
        color: "#fb923c",
        shape: "triangle",
        anchor: [0.84, 0.34],
        properties: { role: "catalog", output: "detections", cadence: "batch" }
      },
      {
        id: "archive",
        label: "Archive",
        name: "Visibility and image archive",
        color: "#475569",
        shape: "circle",
        labelInside: true,
        anchor: [0.82, 0.62],
        media: { kind: "icon", icon: "archive", scale: 0.7, tint_color: "#f8fafc" },
        properties: { role: "storage", output: "query service", cadence: "continuous" }
      },
      {
        id: "tile-service",
        label: "Tile service",
        name: "Public tile service",
        color: "#a61b3f",
        shape: "circle",
        labelInside: true,
        anchor: [0.9, 0.82],
        properties: { role: "distribution", output: "cutouts", cadence: "daily" }
      }
    ],
    edges: [
      { source: "dish-core", target: "correlator", label: "voltages", style: { stroke_color: "#0f766e" } },
      { source: "dish-south", target: "correlator", label: "voltages", style: { stroke_color: "#14b8a6" } },
      { source: "dish-east", target: "correlator", label: "voltages", style: { stroke_color: "#2dd4bf" } },
      { source: "time-ref", target: "correlator", label: "clock", style: { stroke_color: "#2563eb" } },
      { source: "correlator", target: "rfi", label: "visibilities", style: { stroke_width: 3 } },
      { source: "rfi", target: "gain", label: "cleaned", style: { stroke_color: "#2563eb" } },
      { source: "rfi", target: "bandpass", label: "cleaned", style: { stroke_color: "#60a5fa" } },
      { source: "gain", target: "imager", label: "gain table", style: { stroke_color: "#2563eb" } },
      { source: "bandpass", target: "imager", label: "bandpass", style: { stroke_color: "#60a5fa" } },
      { source: "imager", target: "selfcal", label: "model", style: { stroke_color: "#f59e0b" } },
      { source: "selfcal", target: "imager", label: "refine", style: { stroke_color: "#d97706", stroke_width: 3 } },
      { source: "imager", target: "sourcefinder", label: "images", style: { stroke_color: "#fb923c" } },
      { source: "imager", target: "archive", label: "tiles", style: { stroke_color: "#475569" } },
      { source: "sourcefinder", target: "archive", label: "catalog", style: { stroke_color: "#fb923c" } },
      { source: "sourcefinder", target: "tile-service", label: "preview", style: { stroke_color: "#a61b3f" } },
      { source: "archive", target: "tile-service", label: "publish", style: { stroke_color: "#a61b3f", stroke_width: 3 } }
    ]
  },
  exoplanet: {
    kicker: "A loop from survey trigger to public candidate release",
    title: "Exoplanet follow-up loop",
    summary:
      "A candidate-validation mesh that starts with wide-field transit triggers, then branches through vetting, scheduling, spectroscopic checks, and iterative ephemeris refinement.",
    note:
      "This scenario highlights how a network graph can feel procedural without turning into a strict pipeline. The refinement loop and shared transit-fit node create a clean central spine with side branches.",
    facts: [
      { label: "Best for", value: "Shared downstream nodes and iterative refinement." },
      { label: "Primary signals", value: "Transit depth, radial velocity, stellar activity." },
      { label: "Curated lens", value: "A simplified TOI follow-up network." }
    ],
    legend: [
      { label: "Survey triggers", color: "#0f766e" },
      { label: "Decision core", color: "#111827" },
      { label: "Ground follow-up", color: "#d97706" },
      { label: "Archive and release", color: "#a61b3f" }
    ],
    defaults: {
      nodeRadius: 17,
      edgeWidth: 2,
      layoutIterations: 130,
      showLabels: true,
      showArrows: true,
      forceLabelInside: false
    },
    nodes: [
      {
        id: "tess",
        label: "TESS",
        name: "Wide-field transit feed",
        color: "#0f766e",
        shape: "diamond",
        labelInside: true,
        anchor: [0.1, 0.22],
        properties: { role: "trigger", cadence: "30 min", output: "threshold events" }
      },
      {
        id: "quicklook",
        label: "Quicklook",
        name: "Quicklook vetter",
        color: "#14b8a6",
        shape: "circle",
        labelInside: true,
        properties: { role: "screening", output: "TOI shortlist", cadence: "daily" }
      },
      {
        id: "vetting",
        label: "Vetting",
        name: "Candidate vetting board",
        color: "#111827",
        shape: "square",
        labelInside: true,
        style: { radius: 22 },
        properties: { role: "screening", output: "approved follow-up", cadence: "daily" }
      },
      {
        id: "ephemeris",
        label: "Ephemeris",
        name: "Ephemeris solver",
        color: "#2563eb",
        shape: "circle",
        labelInside: true,
        media: { kind: "icon", icon: "planet", scale: 0.72, tint_color: "#eff6ff" },
        properties: { role: "timing", output: "transit windows", cadence: "iterative" }
      },
      {
        id: "scheduler",
        label: "Scheduler",
        name: "Follow-up scheduler",
        color: "#d97706",
        shape: "square",
        labelInside: true,
        style: { radius: 22 },
        properties: { role: "dispatch", output: "queue plans", cadence: "nightly" }
      },
      {
        id: "ground-phot",
        label: "Ground phot",
        name: "Ground photometry network",
        color: "#f59e0b",
        shape: "triangle",
        anchor: [0.84, 0.24],
        properties: { role: "follow-up", output: "high-cadence curves", cadence: "hours" }
      },
      {
        id: "spectrograph",
        label: "Spectrograph",
        name: "Precision spectrograph",
        color: "#fb923c",
        shape: "triangle",
        anchor: [0.84, 0.5],
        properties: { role: "follow-up", output: "RV series", cadence: "nights" }
      },
      {
        id: "activity-model",
        label: "Activity",
        name: "Stellar activity model",
        color: "#7c3aed",
        shape: "circle",
        labelInside: true,
        anchor: [0.62, 0.78],
        properties: { role: "systematics", output: "stellar priors", cadence: "iterative" }
      },
      {
        id: "transit-fit",
        label: "Transit fit",
        name: "Joint transit fit",
        color: "#111827",
        shape: "diamond",
        labelInside: true,
        style: { radius: 22, stroke_color: "#111827", stroke_width: 2 },
        media: {
          kind: "image",
          image_key: "transit-curve",
          fit: "contain",
          scale: 0.82,
          fallback_icon: "planet"
        },
        properties: { role: "inference", output: "planet posteriors", cadence: "iterative" }
      },
      {
        id: "candidate-board",
        label: "Board",
        name: "Candidate review board",
        color: "#a61b3f",
        shape: "square",
        labelInside: true,
        anchor: [0.88, 0.78],
        properties: { role: "approval", output: "validated set", cadence: "weekly" }
      },
      {
        id: "archive",
        label: "Archive",
        name: "Candidate archive",
        color: "#475569",
        shape: "circle",
        labelInside: true,
        anchor: [0.62, 0.92],
        properties: { role: "storage", output: "candidate records", cadence: "continuous" }
      },
      {
        id: "community",
        label: "Release",
        name: "Community release stream",
        color: "#be123c",
        shape: "circle",
        labelInside: true,
        anchor: [0.9, 0.12],
        properties: { role: "distribution", output: "papers and catalogs", cadence: "batched" }
      }
    ],
    edges: [
      { source: "tess", target: "quicklook", label: "threshold", style: { stroke_color: "#0f766e" } },
      { source: "quicklook", target: "vetting", label: "TOI", style: { stroke_color: "#14b8a6" } },
      { source: "vetting", target: "ephemeris", label: "seed window", style: { stroke_color: "#2563eb" } },
      { source: "ephemeris", target: "scheduler", label: "window", style: { stroke_color: "#2563eb" } },
      { source: "scheduler", target: "ground-phot", label: "queue", style: { stroke_color: "#f59e0b" } },
      { source: "scheduler", target: "spectrograph", label: "queue", style: { stroke_color: "#fb923c" } },
      { source: "ground-phot", target: "transit-fit", label: "light curve", style: { stroke_color: "#f59e0b" } },
      { source: "spectrograph", target: "activity-model", label: "RV series", style: { stroke_color: "#fb923c" } },
      { source: "spectrograph", target: "transit-fit", label: "mass prior", style: { stroke_color: "#fb923c" } },
      { source: "activity-model", target: "transit-fit", label: "stellar prior", style: { stroke_color: "#7c3aed" } },
      { source: "transit-fit", target: "candidate-board", label: "posteriors", style: { stroke_width: 3 } },
      { source: "candidate-board", target: "archive", label: "approve", style: { stroke_color: "#a61b3f" } },
      { source: "candidate-board", target: "community", label: "announce", style: { stroke_color: "#be123c" } },
      { source: "archive", target: "ephemeris", label: "refine", style: { stroke_color: "#475569", stroke_width: 3 } },
      { source: "archive", target: "community", label: "release", style: { stroke_color: "#be123c" } }
    ]
  }
};

const APPEARANCE_PRESETS = {
  relay: {
    title: "Relay",
    nodeStyle: { stroke_color: "#e6e9ef", stroke_width: 2, opacity: 0.96 },
    edgeStyle: { stroke_color: "#7b8696", label_visible: true, opacity: 0.8 },
    selectionStyle: { stroke_color: "#d97706", stroke_width: 3, padding: 8, opacity: 0.96 }
  },
  pulse: {
    title: "Pulse",
    nodeStyle: { stroke_color: "#111827", stroke_width: 2, opacity: 0.98 },
    edgeStyle: { stroke_color: "#a61b3f", label_visible: true, opacity: 0.86 },
    selectionStyle: { stroke_color: "#0f766e", stroke_width: 3, padding: 8, opacity: 0.96 }
  },
  quiet: {
    title: "Quiet",
    nodeStyle: { stroke_color: "#cbd5e1", stroke_width: 1, opacity: 0.9 },
    edgeStyle: { stroke_color: "#94a3b8", label_visible: true, opacity: 0.62 },
    selectionStyle: { stroke_color: "#334155", stroke_width: 2, padding: 7, opacity: 0.9 }
  }
};

const state = {
  scenario: "transient",
  preset: "relay",
  pixelRatio: Math.min(window.devicePixelRatio || 1, 3),
  selectedNodeId: null,
  nodeRadius: 18,
  edgeWidth: 2,
  layoutIterations: 140,
  showLabels: true,
  showArrows: true,
  forceLabelInside: false,
  offsetX: 0,
  offsetY: 0,
  sessionHandle: null,
  sessionDirty: true,
  statusKind: "info"
};

const dom = {
  canvas: document.getElementById("network-canvas"),
  statusPanel: document.getElementById("status-panel"),
  scenarioButtons: Array.from(document.querySelectorAll("[data-scenario]")),
  presetButtons: Array.from(document.querySelectorAll("[data-preset]")),
  resetButton: document.getElementById("network-reset"),
  scenarioSelect: document.getElementById("scenario-select"),
  title: document.getElementById("network-title"),
  kicker: document.getElementById("network-kicker"),
  summary: document.getElementById("network-summary"),
  note: document.getElementById("network-note"),
  facts: document.getElementById("network-facts"),
  stats: document.getElementById("network-stats"),
  legend: document.getElementById("network-legend"),
  details: document.getElementById("network-details"),
  nodeRadius: document.getElementById("node-radius"),
  nodeRadiusValue: document.getElementById("node-radius-value"),
  edgeWidth: document.getElementById("edge-width"),
  edgeWidthValue: document.getElementById("edge-width-value"),
  layoutIterations: document.getElementById("layout-iterations"),
  layoutIterationsValue: document.getElementById("layout-iterations-value"),
  pixelRatio: document.getElementById("pixel-ratio"),
  pixelRatioValue: document.getElementById("pixel-ratio-value"),
  showLabels: document.getElementById("show-labels"),
  showArrows: document.getElementById("show-arrows"),
  labelInside: document.getElementById("label-inside")
};

function clone(value) {
  if (globalThis.structuredClone) {
    return globalThis.structuredClone(value);
  }
  return JSON.parse(JSON.stringify(value));
}

function displayName(node) {
  return node.name ?? node.label ?? node.id;
}

function currentScenario() {
  return SCENARIOS[state.scenario];
}

function currentPreset() {
  return APPEARANCE_PRESETS[state.preset];
}

function setStatus(kind, title, body) {
  state.statusKind = kind;
  dom.statusPanel.className = `status-panel status-${kind}`;
  dom.statusPanel.innerHTML = `
    <p class="status-title">${title}</p>
    <p class="status-body">${body}</p>
  `;
}

function scaleSize(value, factor, minimum = 0) {
  if (value == null) {
    return value;
  }
  return Math.max(minimum, Math.round(value * factor));
}

function scaleNodeStyle(style, pixelRatio) {
  if (!style) {
    return null;
  }
  const next = clone(style);
  if (next.radius != null) {
    next.radius = scaleSize(next.radius, pixelRatio, 1);
  }
  if (next.stroke_width != null) {
    next.stroke_width = scaleSize(next.stroke_width, pixelRatio, 0);
  }
  return next;
}

function scaleEdgeStyle(style, pixelRatio) {
  if (!style) {
    return null;
  }
  const next = clone(style);
  if (next.stroke_width != null) {
    next.stroke_width = scaleSize(next.stroke_width, pixelRatio, 0);
  }
  return next;
}

function scaleSelectionStyle(style, pixelRatio) {
  if (!style) {
    return null;
  }
  const next = clone(style);
  if (next.stroke_width != null) {
    next.stroke_width = scaleSize(next.stroke_width, pixelRatio, 0);
  }
  if (next.padding != null) {
    next.padding = scaleSize(next.padding, pixelRatio, 0);
  }
  return next;
}

function syncCanvasSize(force = false) {
  const rect = dom.canvas.getBoundingClientRect();
  const cssWidth = Math.max(320, Math.round(rect.width));
  const cssHeight = Math.max(320, Math.round(rect.height));
  const width = Math.round(cssWidth * state.pixelRatio);
  const height = Math.round(cssHeight * state.pixelRatio);

  if (!force && dom.canvas.width === width && dom.canvas.height === height) {
    return false;
  }

  dom.canvas.width = width;
  dom.canvas.height = height;
  state.sessionDirty = true;
  return true;
}

function eventPoint(event) {
  const rect = dom.canvas.getBoundingClientRect();
  return {
    x: (event.clientX - rect.left) * (dom.canvas.width / rect.width),
    y: (event.clientY - rect.top) * (dom.canvas.height / rect.height)
  };
}

function updateButtonState(buttons, key, attribute) {
  for (const button of buttons) {
    const active = button.dataset[attribute] === key;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-pressed", String(active));
  }
}

function networkStats(scenario) {
  const pinned = scenario.nodes.filter((node) => Array.isArray(node.anchor)).length;
  return {
    nodeCount: scenario.nodes.length,
    edgeCount: scenario.edges.length,
    pinnedCount: pinned,
    layoutIterations: state.layoutIterations
  };
}

function renderScenarioMeta() {
  const scenario = currentScenario();
  const stats = networkStats(scenario);

  dom.kicker.textContent = scenario.kicker;
  dom.title.textContent = scenario.title;
  dom.summary.textContent = scenario.summary;
  dom.note.textContent = scenario.note;

  dom.facts.innerHTML = scenario.facts
    .map(
      (fact) => `
        <div class="info-row">
          <span class="info-label">${fact.label}</span>
          <span class="info-value">${fact.value}</span>
        </div>
      `
    )
    .join("");

  dom.stats.innerHTML = `
    <article class="stat-tile">
      <span class="stat-label">Nodes</span>
      <span class="stat-value">${stats.nodeCount}</span>
    </article>
    <article class="stat-tile">
      <span class="stat-label">Edges</span>
      <span class="stat-value">${stats.edgeCount}</span>
    </article>
    <article class="stat-tile">
      <span class="stat-label">Pinned anchors</span>
      <span class="stat-value">${stats.pinnedCount}</span>
    </article>
    <article class="stat-tile">
      <span class="stat-label">Iterations</span>
      <span class="stat-value">${stats.layoutIterations}</span>
    </article>
  `;

  dom.legend.innerHTML = scenario.legend
    .map(
      (entry) => `
        <li>
          <span class="legend-swatch" style="background:${entry.color}"></span>
          <span>${entry.label}</span>
        </li>
      `
    )
    .join("");

  updateButtonState(dom.scenarioButtons, state.scenario, "scenario");
  updateButtonState(dom.presetButtons, state.preset, "preset");
  dom.scenarioSelect.value = state.scenario;
}

function renderProperties(properties) {
  const entries = Object.entries(properties ?? {});
  if (!entries.length) {
    const empty = document.createElement("p");
    empty.className = "details-empty-copy";
    empty.textContent = "No custom properties on this node.";
    return empty;
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

function connectionNames(nodeId) {
  const scenario = currentScenario();
  const byId = new Map(scenario.nodes.map((node) => [node.id, node]));
  const inbound = [];
  const outbound = [];

  for (const edge of scenario.edges) {
    if (edge.target === nodeId) {
      const from = byId.get(edge.source);
      inbound.push(displayName(from ?? { id: edge.source }));
    }
    if (edge.source === nodeId) {
      const to = byId.get(edge.target);
      outbound.push(displayName(to ?? { id: edge.target }));
    }
  }

  return { inbound, outbound };
}

function renderConnectionGroup(title, names) {
  const wrapper = document.createElement("div");
  wrapper.className = "connection-group";

  const label = document.createElement("span");
  label.className = "detail-key";
  label.textContent = title;

  const list = document.createElement("ul");
  if (!names.length) {
    const item = document.createElement("li");
    item.className = "connection-pill";
    item.textContent = "None";
    list.append(item);
  } else {
    for (const name of names) {
      const item = document.createElement("li");
      item.className = "connection-pill";
      item.textContent = name;
      list.append(item);
    }
  }

  wrapper.append(label, list);
  return wrapper;
}

function renderDetails() {
  const scenario = currentScenario();
  const node = scenario.nodes.find((entry) => entry.id === state.selectedNodeId);

  if (!node) {
    const stack = document.createElement("div");
    stack.className = "selection-stack";

    const heading = document.createElement("h4");
    heading.textContent = "No node selected";

    const body = document.createElement("p");
    body.className = "details-empty-copy";
    body.textContent =
      "Select a node to inspect its neighbors, role, and graph-specific styling context.";

    const presetBlock = document.createElement("div");
    presetBlock.className = "detail-block";
    presetBlock.innerHTML = `
      <span class="detail-key">Active appearance</span>
      <span class="detail-value">${currentPreset().title}</span>
    `;

    stack.append(heading, body, presetBlock);
    dom.details.replaceChildren(stack);
    return;
  }

  const { inbound, outbound } = connectionNames(node.id);
  const stack = document.createElement("div");
  stack.className = "selection-stack";

  const heading = document.createElement("h4");
  heading.textContent = displayName(node);

  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = `ID ${node.id} - in ${inbound.length} - out ${outbound.length}`;

  const renderBlock = document.createElement("div");
  renderBlock.className = "detail-block";
  const mediaLabel = node.media?.kind === "image"
    ? `image (${node.media.image_key})`
    : node.media?.icon
      ? `icon (${node.media.icon})`
      : "none";
  renderBlock.innerHTML = `
    <span class="detail-key">Render treatment</span>
    <span class="detail-value">${node.shape ?? "circle"} node, media ${mediaLabel}${state.forceLabelInside ? ", labels forced inside" : ""}</span>
  `;

  const connections = document.createElement("div");
  connections.className = "connection-grid";
  connections.append(
    renderConnectionGroup("Inbound", inbound),
    renderConnectionGroup("Outbound", outbound)
  );

  stack.append(heading, meta, renderBlock, connections, renderProperties(node.properties));
  dom.details.replaceChildren(stack);
}

function syncControls() {
  dom.nodeRadius.value = String(state.nodeRadius);
  dom.nodeRadiusValue.textContent = `${state.nodeRadius}px`;

  dom.edgeWidth.value = String(state.edgeWidth);
  dom.edgeWidthValue.textContent = `${state.edgeWidth}px`;

  dom.layoutIterations.value = String(state.layoutIterations);
  dom.layoutIterationsValue.textContent = String(state.layoutIterations);

  dom.pixelRatio.value = String(state.pixelRatio);
  dom.pixelRatioValue.textContent = `${state.pixelRatio}x`;

  dom.showLabels.checked = state.showLabels;
  dom.showArrows.checked = state.showArrows;
  dom.labelInside.checked = state.forceLabelInside;
}

function buildSpec() {
  const scenario = currentScenario();
  const preset = currentPreset();
  const pixelRatio = state.pixelRatio;
  const offsetX = Number.isFinite(state.offsetX) ? Math.round(state.offsetX) : 0;
  const offsetY = Number.isFinite(state.offsetY) ? Math.round(state.offsetY) : 0;

  return {
    width: dom.canvas.width,
    height: dom.canvas.height,
    title: scenario.title,
    node_radius: scaleSize(state.nodeRadius, pixelRatio, 1),
    margin: scaleSize(34, pixelRatio, 1),
    offset_x: offsetX,
    offset_y: offsetY,
    pixel_ratio: pixelRatio,
    selected_node_id: state.selectedNodeId,
    layout_iterations: state.layoutIterations,
    show_arrows: state.showArrows,
    show_labels: state.showLabels,
    default_node_style: scaleNodeStyle(preset.nodeStyle, pixelRatio),
    default_edge_style: scaleEdgeStyle(
      { ...preset.edgeStyle, stroke_width: state.edgeWidth },
      pixelRatio
    ),
    selection_style: scaleSelectionStyle(preset.selectionStyle, pixelRatio),
    nodes: scenario.nodes.map((node) => ({
      id: node.id,
      label: node.label,
      name: node.name ?? null,
      color: node.color ?? null,
      x: Array.isArray(node.anchor) ? Math.round(node.anchor[0] * dom.canvas.width) : null,
      y: Array.isArray(node.anchor) ? Math.round(node.anchor[1] * dom.canvas.height) : null,
      shape: node.shape ?? "circle",
      label_inside: state.forceLabelInside ? true : Boolean(node.labelInside),
      style: scaleNodeStyle(node.style, pixelRatio),
      media: node.media ?? null,
      properties: node.properties ?? {}
    })),
    edges: scenario.edges.map((edge) => ({
      source: edge.source,
      target: edge.target,
      label: edge.label ?? null,
      color: edge.color ?? null,
      weight: edge.weight ?? null,
      style: scaleEdgeStyle(edge.style, pixelRatio)
    }))
  };
}

function supportsSessions() {
  return typeof wasm.create_network_session === "function";
}

function destroySession() {
  if (!supportsSessions() || state.sessionHandle == null) {
    state.sessionHandle = null;
    return;
  }

  try {
    wasm.destroy_network_session(state.sessionHandle);
  } finally {
    state.sessionHandle = null;
  }
}

function normalizeSessionOffsets() {
  state.offsetX = Number.isFinite(state.offsetX) ? Math.round(state.offsetX) : 0;
  state.offsetY = Number.isFinite(state.offsetY) ? Math.round(state.offsetY) : 0;
}

function syncSession() {
  if (!supportsSessions()) {
    state.sessionHandle = null;
    state.sessionDirty = false;
    return;
  }

  normalizeSessionOffsets();
  destroySession();
  state.sessionHandle = wasm.create_network_session("network-canvas", buildSpec());
  state.sessionDirty = false;
}

function render() {
  syncCanvasSize();

  try {
    if (supportsSessions()) {
      if (state.sessionHandle == null || state.sessionDirty) {
        syncSession();
      }
      wasm.render_network_session(state.sessionHandle);
    } else {
      wasm.render_network("network-canvas", buildSpec());
      state.sessionDirty = false;
    }

    if (state.statusKind !== "success") {
      setStatus("success", "WASM ready.", "Drag to pan, scroll to zoom, and click a node to inspect inbound and outbound links.");
    }
  } catch (error) {
    console.error("[network] render failed", error);
    setStatus("error", "Network render failed.", error?.message ?? String(error));
  }
}

function applyScenario(nextScenario) {
  const scenario = SCENARIOS[nextScenario];
  state.scenario = nextScenario;
  state.selectedNodeId = null;
  state.offsetX = 0;
  state.offsetY = 0;
  state.nodeRadius = scenario.defaults.nodeRadius;
  state.edgeWidth = scenario.defaults.edgeWidth;
  state.layoutIterations = scenario.defaults.layoutIterations;
  state.showLabels = scenario.defaults.showLabels;
  state.showArrows = scenario.defaults.showArrows;
  state.forceLabelInside = scenario.defaults.forceLabelInside;
  state.sessionDirty = true;

  syncControls();
  renderScenarioMeta();
  renderDetails();
  render();
}

function resetView() {
  applyScenario(state.scenario);
}

function attachControls() {
  for (const button of dom.scenarioButtons) {
    button.addEventListener("click", () => applyScenario(button.dataset.scenario));
  }

  for (const button of dom.presetButtons) {
    button.addEventListener("click", () => {
      state.preset = button.dataset.preset;
      state.sessionDirty = true;
      renderScenarioMeta();
      renderDetails();
      render();
    });
  }

  dom.scenarioSelect.addEventListener("change", () => {
    applyScenario(dom.scenarioSelect.value);
  });

  dom.nodeRadius.addEventListener("input", () => {
    state.nodeRadius = Number(dom.nodeRadius.value);
    dom.nodeRadiusValue.textContent = `${state.nodeRadius}px`;
    state.sessionDirty = true;
    render();
  });

  dom.edgeWidth.addEventListener("input", () => {
    state.edgeWidth = Number(dom.edgeWidth.value);
    dom.edgeWidthValue.textContent = `${state.edgeWidth}px`;
    state.sessionDirty = true;
    render();
  });

  dom.layoutIterations.addEventListener("input", () => {
    state.layoutIterations = Number(dom.layoutIterations.value);
    dom.layoutIterationsValue.textContent = String(state.layoutIterations);
    state.sessionDirty = true;
    renderScenarioMeta();
    render();
  });

  dom.pixelRatio.addEventListener("input", () => {
    const next = Number(dom.pixelRatio.value);
    const ratio = next / state.pixelRatio;
    state.pixelRatio = next;
    state.offsetX = Math.round(state.offsetX * ratio);
    state.offsetY = Math.round(state.offsetY * ratio);
    dom.pixelRatioValue.textContent = `${state.pixelRatio}x`;
    syncCanvasSize(true);
    state.sessionDirty = true;
    render();
  });

  dom.showLabels.addEventListener("change", () => {
    state.showLabels = dom.showLabels.checked;
    state.sessionDirty = true;
    render();
  });

  dom.showArrows.addEventListener("change", () => {
    state.showArrows = dom.showArrows.checked;
    state.sessionDirty = true;
    render();
  });

  dom.labelInside.addEventListener("change", () => {
    state.forceLabelInside = dom.labelInside.checked;
    state.sessionDirty = true;
    renderDetails();
    render();
  });

  dom.resetButton.addEventListener("click", resetView);
}

function setSelection(nodeId) {
  state.selectedNodeId = nodeId;
  if (supportsSessions() && state.sessionHandle != null) {
    wasm.set_network_selection(state.sessionHandle, nodeId ?? undefined);
  }
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
    if (!drag.active) {
      return;
    }

    const point = eventPoint(event);
    const dx = point.x - drag.last.x;
    const dy = point.y - drag.last.y;

    if (dx === 0 && dy === 0) {
      return;
    }

    drag.last = point;
    drag.moved =
      drag.moved || Math.hypot(point.x - drag.start.x, point.y - drag.start.y) > 4;
    state.offsetX += dx;
    state.offsetY += dy;

    if (supportsSessions() && state.sessionHandle != null && !state.sessionDirty) {
      wasm.pan_network_session(state.sessionHandle, dx, dy);
      wasm.render_network_session(state.sessionHandle);
      return;
    }

    render();
  });

  function finishPointer(event) {
    if (!drag.active) {
      return;
    }

    const point = eventPoint(event);
    const wasClick =
      !drag.moved || Math.hypot(point.x - drag.start.x, point.y - drag.start.y) <= 4;

    drag.active = false;
    dom.canvas.classList.remove("is-dragging");

    if (dom.canvas.hasPointerCapture(event.pointerId)) {
      dom.canvas.releasePointerCapture(event.pointerId);
    }

    if (!wasClick) {
      return;
    }

    try {
      let hit = null;
      if (supportsSessions() && state.sessionHandle != null && !state.sessionDirty) {
        hit = wasm.pick_network_node_session(state.sessionHandle, point.x, point.y);
      } else {
        hit = wasm.pick_network_node(buildSpec(), point.x, point.y);
      }
      setSelection(hit?.node_id ?? null);
      renderDetails();
      render();
    } catch (error) {
      console.error("[network] pick failed", error);
    }
  }

  dom.canvas.addEventListener("pointerup", finishPointer);
  dom.canvas.addEventListener("pointercancel", finishPointer);

  dom.canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const point = eventPoint(event);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;

      if (supportsSessions()) {
        if (state.sessionHandle == null || state.sessionDirty) {
          syncSession();
        }
        wasm.zoom_network_session(state.sessionHandle, point.x, point.y, factor);
        wasm.render_network_session(state.sessionHandle);
        return;
      }

      render();
    },
    { passive: false }
  );
}

async function boot() {
  try {
    await init();
    await preloadGraphImages();
  } catch (error) {
    console.error("[network] wasm init failed", error);
    setStatus("error", "Unable to load WASM.", error?.message ?? String(error));
    return;
  }

  state.pixelRatio = Math.max(1, Math.min(3, Math.round(state.pixelRatio * 2) / 2));

  syncControls();
  attachControls();
  attachCanvasInteractions();
  applyScenario(state.scenario);

  let resizeFrame = null;
  const observer = new ResizeObserver(() => {
    if (resizeFrame != null) {
      cancelAnimationFrame(resizeFrame);
    }
    resizeFrame = requestAnimationFrame(() => {
      resizeFrame = null;
      syncCanvasSize(true);
      state.sessionDirty = true;
      render();
    });
  });

  observer.observe(dom.canvas);
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
