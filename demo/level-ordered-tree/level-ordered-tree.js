import init, * as wasm from "../../pkg/navi_plot_wasm.js";

const MIN_SCREEN_ZOOM = 0.25;
const MAX_SCREEN_ZOOM = 8.0;

const DEMO_IMAGE_SOURCES = {
  "planetary-nebula": new URL("../assets/planetary-nebula.svg", import.meta.url).href
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
        console.warn(`[level-ordered-tree] image preload failed for ${key}`, error);
      }
    })
  );
}

const SCENARIOS = {
  handoff: {
    kicker: "Rooted handoff with explicit left-to-right order",
    title: "Night follow-up order",
    summary:
      "A fixed root fans into one coordination band and one delivery band. Siblings keep a left-to-right sequence, and only siblings under the same parent share adjacency links.",
    note:
      "Use this shape when a rooted flow still needs visible sibling order without implying cross-parent ties. Each family keeps its own sequence while the hierarchy stays readable.",
    facts: [
      { label: "Root anchor", value: "Night program board" },
      { label: "Band logic", value: "Only siblings that share a parent are chained in order." },
      { label: "Visual mix", value: "Icons, one registered image node, and optional external order badges." }
    ],
    legend: [
      { label: "Root anchor", color: "#111827" },
      { label: "Coordination band", color: "#0f766e" },
      { label: "Delivery band", color: "#d97706" },
      { label: "Order links", color: "#64748b" }
    ],
    defaults: {
      nodeRadius: 18,
      edgeWidth: 2,
      showOrderBadges: false
    },
    root: {
      id: "night-board",
      label: "Night board",
      name: "Night program board",
      color: "#111827",
      shape: "square",
      labelInside: true,
      anchor: [0.5, 0.14],
      style: { radius: 24, stroke_color: "#111827", stroke_width: 2 },
      media: { kind: "icon", icon: "broker", scale: 0.72, tint_color: "#f8fafc" },
      properties: {
        role: "root anchor",
        cadence: "continuous",
        output: "ordered follow-up lanes"
      }
    },
    bands: [
      {
        id: "coordination",
        label: "Coordination band",
        yAnchor: 0.38,
        xStart: 0.18,
        xEnd: 0.82,
        orderColor: "#64748b",
        nodes: [
          {
            id: "alert-lane",
            label: "Alerts",
            name: "Alert intake lane",
            orderIndex: 1,
            parentId: "night-board",
            color: "#0f766e",
            shape: "diamond",
            labelInside: true,
            media: { kind: "icon", icon: "alert", scale: 0.66, tint_color: "#ecfdf5" },
            properties: {
              role: "intake",
              cadence: "minutes",
              output: "sky packets"
            }
          },
          {
            id: "broker-lane",
            label: "Broker",
            name: "Broker ranking lane",
            orderIndex: 2,
            parentId: "night-board",
            color: "#2563eb",
            shape: "square",
            labelInside: true,
            media: { kind: "icon", icon: "database", scale: 0.68, tint_color: "#eff6ff" },
            properties: {
              role: "ranking",
              cadence: "seconds",
              output: "priority sets"
            }
          },
          {
            id: "queue-lane",
            label: "Queue",
            name: "Observation queue lane",
            orderIndex: 3,
            parentId: "night-board",
            color: "#d97706",
            shape: "square",
            labelInside: true,
            media: { kind: "icon", icon: "camera", scale: 0.68, tint_color: "#fff7ed" },
            properties: {
              role: "dispatch",
              cadence: "hours",
              output: "instrument slots"
            }
          },
          {
            id: "archive-lane",
            label: "Archive",
            name: "Archive and release lane",
            orderIndex: 4,
            parentId: "night-board",
            color: "#a61b3f",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "archive", scale: 0.7, tint_color: "#fff1f2" },
            properties: {
              role: "delivery",
              cadence: "continuous",
              output: "release bundles"
            }
          }
        ]
      },
      {
        id: "delivery",
        label: "Delivery band",
        yAnchor: 0.72,
        xStart: 0.12,
        xEnd: 0.88,
        orderColor: "#9a3412",
        nodes: [
          {
            id: "sky-map",
            label: "Sky map",
            name: "Localization sky map",
            orderIndex: 1,
            parentId: "alert-lane",
            color: "#14b8a6",
            shape: "circle",
            media: { kind: "icon", icon: "telescope", scale: 0.64, tint_color: "#ecfeff" },
            properties: {
              role: "packet",
              cadence: "minutes",
              output: "pointing window"
            }
          },
          {
            id: "ranked-set",
            label: "Scores",
            name: "Ranked candidate set",
            orderIndex: 1,
            parentId: "broker-lane",
            color: "#475569",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "database", scale: 0.62, tint_color: "#f8fafc" },
            properties: {
              role: "candidate pack",
              cadence: "seconds",
              output: "ranked shortlist"
            }
          },
          {
            id: "field-cutout",
            label: "Cutout",
            name: "Deep-field cutout pack",
            orderIndex: 1,
            parentId: "queue-lane",
            color: "#0f766e",
            shape: "circle",
            style: { radius: 20 },
            media: {
              kind: "image",
              image_key: "planetary-nebula",
              fit: "cover",
              scale: 0.88,
              fallback_icon: "camera"
            },
            properties: {
              role: "preview tile",
              cadence: "hours",
              output: "field pack"
            }
          },
          {
            id: "spectra-slot",
            label: "Spectra",
            name: "Spectroscopy slot",
            orderIndex: 2,
            parentId: "queue-lane",
            color: "#fb923c",
            shape: "triangle",
            labelInside: true,
            media: { kind: "icon", icon: "spectrograph", scale: 0.7, tint_color: "#fff7ed" },
            properties: {
              role: "follow-up",
              cadence: "same night",
              output: "queued spectra"
            }
          },
          {
            id: "release-pack",
            label: "Release",
            name: "Release bundle",
            orderIndex: 1,
            parentId: "archive-lane",
            color: "#be123c",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "star", scale: 0.62, tint_color: "#fff1f2" },
            properties: {
              role: "distribution",
              cadence: "batched",
              output: "community bundle"
            }
          }
        ]
      }
    ]
  },
  astrobook: {
    kicker: "Textbook chapter map with local section order",
    title: "Astrobook chapter map",
    summary:
      "A single chapter anchors ordered sections and ordered subsubsections. Every section shares one level color, and every subsubsection shares another.",
    note:
      "This mirrors a textbook spread at chapter scope: the chapter stays as a hidden anchor, the first band is sections, and the second band is subsubsections grouped under their own section.",
    hideRootNode: true,
    facts: [
      { label: "Root anchor", value: "Hidden chapter anchor for Astrobook Chapter 7" },
      { label: "Band logic", value: "Sections and subsubsections only chain inside their own parent family." },
      { label: "Level color", value: "All sections share one color, and all subsubsections share one color." }
    ],
    legend: [
      { label: "Hidden chapter anchor", color: "#111827" },
      { label: "Sections", color: "#0f766e" },
      { label: "Subsubsections", color: "#d97706" },
      { label: "Order links", color: "#64748b" }
    ],
    defaults: {
      nodeRadius: 18,
      edgeWidth: 2,
      showOrderBadges: true
    },
    root: {
      id: "astrobook-ch7",
      label: "Chapter 7",
      name: "Astrobook Chapter 7: Stellar Nurseries",
      color: "#111827",
      shape: "square",
      labelInside: true,
      anchor: [0.5, 0.14],
      style: { radius: 24, stroke_color: "#111827", stroke_width: 2 },
      media: { kind: "icon", icon: "star", scale: 0.68, tint_color: "#f8fafc" },
      properties: {
        role: "chapter anchor",
        pages: "212-263",
        theme: "stellar nurseries"
      }
    },
    bands: [
      {
        id: "sections",
        label: "Sections",
        yAnchor: 0.38,
        xStart: 0.16,
        xEnd: 0.84,
        orderColor: "#64748b",
        nodes: [
          {
            id: "sec-clouds",
            label: "7.1 Clouds",
            name: "7.1 Molecular clouds",
            orderIndex: 1,
            parentId: "astrobook-ch7",
            color: "#0f766e",
            shape: "square",
            labelInside: true,
            media: { kind: "icon", icon: "galaxy", scale: 0.68, tint_color: "#ecfdf5" },
            properties: {
              role: "section",
              focus: "cold gas reservoirs",
              span: "pages 214-225"
            }
          },
          {
            id: "sec-collapse",
            label: "7.2 Collapse",
            name: "7.2 Gravitational collapse",
            orderIndex: 2,
            parentId: "astrobook-ch7",
            color: "#0f766e",
            shape: "square",
            labelInside: true,
            media: { kind: "icon", icon: "alert", scale: 0.64, tint_color: "#ecfdf5" },
            properties: {
              role: "section",
              focus: "instability and fragmentation",
              span: "pages 226-238"
            }
          },
          {
            id: "sec-feedback",
            label: "7.3 Feedback",
            name: "7.3 Protostellar feedback",
            orderIndex: 3,
            parentId: "astrobook-ch7",
            color: "#0f766e",
            shape: "square",
            labelInside: true,
            media: { kind: "icon", icon: "telescope", scale: 0.66, tint_color: "#ecfdf5" },
            properties: {
              role: "section",
              focus: "outflows and radiation",
              span: "pages 239-250"
            }
          },
          {
            id: "sec-surveys",
            label: "7.4 Surveys",
            name: "7.4 Atlas and survey notes",
            orderIndex: 4,
            parentId: "astrobook-ch7",
            color: "#0f766e",
            shape: "square",
            labelInside: true,
            media: { kind: "icon", icon: "database", scale: 0.66, tint_color: "#ecfdf5" },
            properties: {
              role: "section",
              focus: "reference atlases",
              span: "pages 251-263"
            }
          }
        ]
      },
      {
        id: "subsubsections",
        label: "Subsubsections",
        yAnchor: 0.72,
        xStart: 0.12,
        xEnd: 0.88,
        orderColor: "#8f5a19",
        nodes: [
          {
            id: "cloud-cores",
            label: "7.1.1 Cores",
            name: "7.1.1 Dense cores",
            orderIndex: 1,
            parentId: "sec-clouds",
            color: "#d97706",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "moon", scale: 0.62, tint_color: "#fff7ed" },
            properties: {
              role: "subsubsection",
              focus: "core chemistry",
              pages: "214-219"
            }
          },
          {
            id: "cloud-dust",
            label: "7.1.2 Dust",
            name: "7.1.2 Dust lanes",
            orderIndex: 2,
            parentId: "sec-clouds",
            color: "#d97706",
            shape: "circle",
            media: { kind: "icon", icon: "camera", scale: 0.62, tint_color: "#fff7ed" },
            properties: {
              role: "subsubsection",
              focus: "extinction and maps",
              pages: "220-225"
            }
          },
          {
            id: "collapse-jeans",
            label: "7.2.1 Jeans",
            name: "7.2.1 Jeans criteria",
            orderIndex: 1,
            parentId: "sec-collapse",
            color: "#d97706",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "alert", scale: 0.6, tint_color: "#fff7ed" },
            properties: {
              role: "subsubsection",
              focus: "critical mass",
              pages: "226-232"
            }
          },
          {
            id: "collapse-fragment",
            label: "7.2.2 Fragment",
            name: "7.2.2 Fragmentation",
            orderIndex: 2,
            parentId: "sec-collapse",
            color: "#d97706",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "star", scale: 0.6, tint_color: "#fff7ed" },
            properties: {
              role: "subsubsection",
              focus: "core splitting",
              pages: "233-238"
            }
          },
          {
            id: "feedback-shells",
            label: "7.3.1 Shells",
            name: "7.3.1 Radiation shells",
            orderIndex: 1,
            parentId: "sec-feedback",
            color: "#d97706",
            shape: "circle",
            style: { radius: 20 },
            media: {
              kind: "image",
              image_key: "planetary-nebula",
              fit: "cover",
              scale: 0.88,
              fallback_icon: "camera"
            },
            properties: {
              role: "subsubsection",
              focus: "ionized bubbles",
              pages: "239-244"
            }
          },
          {
            id: "feedback-outflows",
            label: "7.3.2 Outflows",
            name: "7.3.2 Bipolar outflows",
            orderIndex: 2,
            parentId: "sec-feedback",
            color: "#d97706",
            shape: "triangle",
            labelInside: true,
            media: { kind: "icon", icon: "spectrograph", scale: 0.7, tint_color: "#fff7ed" },
            properties: {
              role: "subsubsection",
              focus: "jet signatures",
              pages: "245-250"
            }
          },
          {
            id: "survey-alma",
            label: "7.4.1 ALMA",
            name: "7.4.1 ALMA atlas",
            orderIndex: 1,
            parentId: "sec-surveys",
            color: "#d97706",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "dish", scale: 0.66, tint_color: "#fff7ed" },
            properties: {
              role: "subsubsection",
              focus: "millimeter maps",
              pages: "251-257"
            }
          },
          {
            id: "survey-jwst",
            label: "7.4.2 JWST",
            name: "7.4.2 JWST reference plates",
            orderIndex: 2,
            parentId: "sec-surveys",
            color: "#d97706",
            shape: "circle",
            labelInside: true,
            media: { kind: "icon", icon: "camera", scale: 0.64, tint_color: "#fff7ed" },
            properties: {
              role: "subsubsection",
              focus: "infrared plates",
              pages: "258-263"
            }
          }
        ]
      }
    ]
  }
};

const APPEARANCE_PRESETS = {
  relay: {
    title: "Relay",
    nodeStyle: { stroke_color: "#e4e7ec", stroke_width: 2, opacity: 0.97 },
    edgeStyle: { stroke_color: "#7b8696", label_visible: false, opacity: 0.78 },
    selectionStyle: { stroke_color: "#0f766e", stroke_width: 3, padding: 8, opacity: 0.96 }
  },
  ember: {
    title: "Ember",
    nodeStyle: { stroke_color: "#1f2937", stroke_width: 2, opacity: 0.98 },
    edgeStyle: { stroke_color: "#b45309", label_visible: false, opacity: 0.82 },
    selectionStyle: { stroke_color: "#a61b3f", stroke_width: 3, padding: 8, opacity: 0.96 }
  },
  quiet: {
    title: "Quiet",
    nodeStyle: { stroke_color: "#cbd5cf", stroke_width: 1, opacity: 0.92 },
    edgeStyle: { stroke_color: "#94a3b8", label_visible: false, opacity: 0.58 },
    selectionStyle: { stroke_color: "#334155", stroke_width: 2, padding: 7, opacity: 0.9 }
  }
};

const ORDERED_TREE_INDEXES = Object.fromEntries(
  Object.entries(SCENARIOS).map(([key, scenario]) => [key, createOrderedTreeIndex(scenario)])
);

const state = {
  scenario: "handoff",
  preset: "relay",
  pixelRatio: Math.min(window.devicePixelRatio || 1, 3),
  selectedNodeId: null,
  nodeRadius: SCENARIOS.handoff.defaults.nodeRadius,
  edgeWidth: SCENARIOS.handoff.defaults.edgeWidth,
  showOrderBadges: SCENARIOS.handoff.defaults.showOrderBadges,
  offsetX: 0,
  offsetY: 0,
  zoom: 1,
  sessionHandle: null,
  sessionDirty: true,
  statusKind: "info",
  layoutSnapshot: new Map()
};

const dom = {
  canvas: document.getElementById("level-ordered-tree-canvas"),
  badgeLayer: document.getElementById("order-badge-layer"),
  statusPanel: document.getElementById("status-panel"),
  scenarioButtons: Array.from(document.querySelectorAll("[data-scenario]")),
  presetButtons: Array.from(document.querySelectorAll("[data-preset]")),
  resetButton: document.getElementById("ordered-tree-reset"),
  scenarioSelect: document.getElementById("scenario-select"),
  title: document.getElementById("ordered-tree-title"),
  kicker: document.getElementById("ordered-tree-kicker"),
  summary: document.getElementById("ordered-tree-summary"),
  note: document.getElementById("ordered-tree-note"),
  facts: document.getElementById("ordered-tree-facts"),
  stats: document.getElementById("ordered-tree-stats"),
  legend: document.getElementById("ordered-tree-legend"),
  details: document.getElementById("ordered-tree-details"),
  nodeRadius: document.getElementById("node-radius"),
  nodeRadiusValue: document.getElementById("node-radius-value"),
  edgeWidth: document.getElementById("edge-width"),
  edgeWidthValue: document.getElementById("edge-width-value"),
  pixelRatio: document.getElementById("pixel-ratio"),
  pixelRatioValue: document.getElementById("pixel-ratio-value"),
  showOrderBadges: document.getElementById("show-order-badges")
};

function clone(value) {
  if (globalThis.structuredClone) {
    return globalThis.structuredClone(value);
  }
  return JSON.parse(JSON.stringify(value));
}

function displayName(node) {
  return node?.name ?? node?.label ?? node?.id ?? "Unknown node";
}

function currentScenario() {
  return SCENARIOS[state.scenario];
}

function currentIndex() {
  return ORDERED_TREE_INDEXES[state.scenario];
}

function isHiddenRootNodeId(nodeId) {
  return Boolean(currentScenario().hideRootNode) && nodeId === currentIndex().root.id;
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

function validateOrderedTree(data) {
  if (!data?.root?.id) {
    throw new Error("ordered tree root is required");
  }
  if (!Array.isArray(data.bands) || data.bands.length !== 2) {
    throw new Error("level-ordered tree requires exactly two descendant bands");
  }
}

function createOrderedTreeIndex(data) {
  validateOrderedTree(data);

  const root = {
    ...data.root,
    isRoot: true,
    level: 0,
    bandId: null,
    bandLabel: "Root"
  };

  const byId = new Map([[root.id, root]]);
  const children = new Map([[root.id, []]]);
  const bands = [];
  const neighbors = new Map();

  for (const [bandIndex, band] of data.bands.entries()) {
    if (!Array.isArray(band.nodes) || band.nodes.length === 0) {
      throw new Error(`band ${band.id} must contain at least one node`);
    }

    const level = bandIndex + 1;
    const preparedNodes = band.nodes.map((node) => ({
      ...node,
      level,
      bandId: band.id,
      bandLabel: band.label,
      parentId: node.parentId ?? (level === 1 ? root.id : null)
    }));

    for (const node of preparedNodes) {
      if (!Number.isInteger(node.orderIndex) || node.orderIndex < 1) {
        throw new Error(`node ${node.id} must use a positive integer orderIndex`);
      }
      if (byId.has(node.id)) {
        throw new Error(`duplicate node id ${node.id}`);
      }
      byId.set(node.id, node);
      children.set(node.id, []);
    }

    const preparedBand = { ...band, level, nodes: preparedNodes, siblingGroups: [] };
    bands.push(preparedBand);
  }

  const hierarchyEdges = [];
  const orderEdges = [];

  for (const band of bands) {
    const siblingGroups = new Map();

    for (const node of band.nodes) {
      if (!node.parentId || !byId.has(node.parentId)) {
        throw new Error(`node ${node.id} references unknown parent ${node.parentId}`);
      }

      const parent = byId.get(node.parentId);
      if (band.level === 1 && node.parentId !== root.id) {
        throw new Error(`level 1 node ${node.id} must attach to the root`);
      }
      if (band.level === 2 && parent?.level !== 1) {
        throw new Error(`level 2 node ${node.id} must attach to a level 1 node`);
      }

      hierarchyEdges.push({
        source: node.parentId,
        target: node.id,
        kind: "hierarchy",
        level: band.level
      });
      children.get(node.parentId).push(node.id);

      const siblings = siblingGroups.get(node.parentId) ?? [];
      siblings.push(node);
      siblingGroups.set(node.parentId, siblings);
    }

    band.siblingGroups = Array.from(siblingGroups.entries()).map(([parentId, siblings]) => {
      const orderedSiblings = [...siblings].sort((left, right) => left.orderIndex - right.orderIndex);
      const seenOrders = new Set();

      for (const node of orderedSiblings) {
        if (seenOrders.has(node.orderIndex)) {
          throw new Error(
            `siblings under ${parentId} contain duplicate order index ${node.orderIndex}`
          );
        }
        seenOrders.add(node.orderIndex);
      }

      orderedSiblings.forEach((node, index) => {
        neighbors.set(node.id, {
          bandId: band.id,
          bandLabel: band.label,
          parentId,
          size: orderedSiblings.length,
          leftId: index > 0 ? orderedSiblings[index - 1].id : null,
          rightId: index < orderedSiblings.length - 1 ? orderedSiblings[index + 1].id : null
        });
      });

      for (let index = 0; index < orderedSiblings.length - 1; index += 1) {
        orderEdges.push({
          source: orderedSiblings[index].id,
          target: orderedSiblings[index + 1].id,
          kind: "order",
          level: band.level,
          bandId: band.id,
          parentId
        });
      }

      return { parentId, nodes: orderedSiblings };
    });
  }

  for (const node of byId.values()) {
    if (node.isRoot) {
      continue;
    }
    if (!neighbors.has(node.id)) {
      neighbors.set(node.id, {
        bandId: node.bandId,
        bandLabel: node.bandLabel,
        parentId: node.parentId,
        size: 1,
        leftId: null,
        rightId: null
      });
    }
  }

  return {
    root,
    bands,
    byId,
    children,
    neighbors,
    nodes: [root, ...bands.flatMap((band) => band.nodes)],
    orderedNodes: bands.flatMap((band) => band.nodes),
    hierarchyEdges,
    orderEdges
  };
}

function computeNodePositions(width, height) {
  const scenario = currentScenario();
  const index = currentIndex();
  const positions = new Map();
  positions.set(index.root.id, {
    x: Math.round(scenario.root.anchor[0] * width),
    y: Math.round(scenario.root.anchor[1] * height)
  });

  for (const band of index.bands) {
    if (band.level === 1) {
      const orderedRootChildren = [...band.nodes].sort((left, right) => left.orderIndex - right.orderIndex);
      const count = orderedRootChildren.length;
      orderedRootChildren.forEach((node, index) => {
        const ratio = count === 1 ? 0.5 : index / (count - 1);
        const x = band.xStart + (band.xEnd - band.xStart) * ratio;
        positions.set(node.id, {
          x: Math.round(x * width),
          y: Math.round(band.yAnchor * height)
        });
      });
      continue;
    }

    const baseGap = Math.min(0.11, Math.max(0.06, (band.xEnd - band.xStart) / (band.nodes.length + 2)));
    const orderedGroups = [...band.siblingGroups].sort((left, right) => {
      const leftParent = positions.get(left.parentId)?.x ?? 0;
      const rightParent = positions.get(right.parentId)?.x ?? 0;
      return leftParent - rightParent;
    });

    for (const group of orderedGroups) {
      const parentPosition = positions.get(group.parentId);
      const parentRatio = parentPosition ? parentPosition.x / width : 0.5;
      const span = (group.nodes.length - 1) * baseGap;
      const start = Math.min(
        Math.max(parentRatio - span / 2, band.xStart),
        band.xEnd - span
      );

      group.nodes.forEach((node, index) => {
        const x = group.nodes.length === 1 ? parentRatio : start + index * baseGap;
        positions.set(node.id, {
          x: Math.round(Math.min(band.xEnd, Math.max(band.xStart, x)) * width),
          y: Math.round(band.yAnchor * height)
        });
      });
    }
  }

  return positions;
}

function orderedTreeStats() {
  const index = currentIndex();
  const hiddenRootEdgeCount = currentScenario().hideRootNode
    ? index.hierarchyEdges.filter((edge) => edge.source === index.root.id).length
    : 0;
  return {
    nodeCount: index.nodes.length - (currentScenario().hideRootNode ? 1 : 0),
    edgeCount: index.hierarchyEdges.length + index.orderEdges.length - hiddenRootEdgeCount,
    orderedSlots: index.orderedNodes.length,
    badgeMode: state.showOrderBadges ? "On" : "Off"
  };
}

function updateButtonState(buttons, key, attribute) {
  for (const button of buttons) {
    const active = button.dataset[attribute] === key;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-pressed", String(active));
  }
}

function renderMeta() {
  const scenario = currentScenario();
  const stats = orderedTreeStats();

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
      <span class="stat-label">Links</span>
      <span class="stat-value">${stats.edgeCount}</span>
    </article>
    <article class="stat-tile">
      <span class="stat-label">Ordered slots</span>
      <span class="stat-value">${stats.orderedSlots}</span>
    </article>
    <article class="stat-tile">
      <span class="stat-label">Badges</span>
      <span class="stat-value">${stats.badgeMode}</span>
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

function relationNames(node) {
  const index = currentIndex();
  const parent = node.parentId ? index.byId.get(node.parentId) : null;
  const childNames = (index.children.get(node.id) ?? []).map((childId) =>
    displayName(index.byId.get(childId))
  );
  const bandNeighbors = index.neighbors.get(node.id);
  const adjacentNames = [];
  if (bandNeighbors?.leftId) {
    adjacentNames.push(displayName(index.byId.get(bandNeighbors.leftId)));
  }
  if (bandNeighbors?.rightId) {
    adjacentNames.push(displayName(index.byId.get(bandNeighbors.rightId)));
  }

  return {
    parentName: parent ? displayName(parent) : null,
    childNames,
    adjacentNames,
    bandNeighbors
  };
}

function renderDetails() {
  const index = currentIndex();
  const node = index.byId.get(state.selectedNodeId);

  if (!node) {
    const stack = document.createElement("div");
    stack.className = "selection-stack";

    const heading = document.createElement("h4");
    heading.textContent = "No node selected";

    const body = document.createElement("p");
    body.className = "details-empty-copy";
    body.textContent =
      "Select a node to inspect its order slot, parent branch, and sibling neighbors.";

    const presetBlock = document.createElement("div");
    presetBlock.className = "detail-block";
    presetBlock.innerHTML = `
      <span class="detail-key">Active appearance</span>
      <span class="detail-value">${currentPreset().title} preset, order badges ${state.showOrderBadges ? "on" : "off"}</span>
    `;

    stack.append(heading, body, presetBlock);
    dom.details.replaceChildren(stack);
    return;
  }

  const { parentName, childNames, adjacentNames, bandNeighbors } = relationNames(node);
  const stack = document.createElement("div");
  stack.className = "selection-stack";

  const heading = document.createElement("h4");
  heading.textContent = displayName(node);

  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent = node.isRoot
    ? "Root anchor"
    : `Level ${node.level} - order ${node.orderIndex} of ${bandNeighbors.size}`;

  const orderBlock = document.createElement("div");
  orderBlock.className = "detail-block";
  orderBlock.innerHTML = `
    <span class="detail-key">Order role</span>
    <span class="detail-value">${node.isRoot ? "Anchors the ordered handoff and feeds the first descendant band." : `${node.bandLabel}, sibling slot ${node.orderIndex} under ${parentName ?? "the root"}.`}</span>
  `;

  const mediaLabel = node.media?.kind === "image"
    ? `image (${node.media.image_key})`
    : node.media?.icon
      ? `icon (${node.media.icon})`
      : "none";

  const renderBlock = document.createElement("div");
  renderBlock.className = "detail-block";
  renderBlock.innerHTML = `
    <span class="detail-key">Render treatment</span>
    <span class="detail-value">${node.shape ?? "circle"} node, media ${mediaLabel}</span>
  `;

  const connections = document.createElement("div");
  connections.className = "connection-grid";
  connections.append(
    renderConnectionGroup("Parent", parentName ? [parentName] : []),
    renderConnectionGroup("Children", childNames),
    renderConnectionGroup("Adjacent", adjacentNames)
  );

  stack.append(heading, meta, orderBlock, renderBlock, connections, renderProperties(node.properties));
  dom.details.replaceChildren(stack);
}

function syncControls() {
  dom.scenarioSelect.value = state.scenario;
  dom.nodeRadius.value = String(state.nodeRadius);
  dom.nodeRadiusValue.textContent = `${state.nodeRadius}px`;

  dom.edgeWidth.value = String(state.edgeWidth);
  dom.edgeWidthValue.textContent = `${state.edgeWidth}px`;

  dom.pixelRatio.value = String(state.pixelRatio);
  dom.pixelRatioValue.textContent = `${state.pixelRatio}x`;

  dom.showOrderBadges.checked = state.showOrderBadges;
}

function invalidateSession() {
  state.sessionDirty = true;
  state.zoom = 1;
}

function normalizeSessionOffsets() {
  state.offsetX = Number.isFinite(state.offsetX) ? Math.round(state.offsetX) : 0;
  state.offsetY = Number.isFinite(state.offsetY) ? Math.round(state.offsetY) : 0;
}

function hierarchyEdgeStyle(edge, pixelRatio) {
  if (currentScenario().hideRootNode && edge.source === currentIndex().root.id) {
    return scaleEdgeStyle(
      {
        stroke_width: 0,
        opacity: 0,
        arrow_visible: false,
        label_visible: false
      },
      pixelRatio
    );
  }

  const target = currentIndex().byId.get(edge.target);
  return scaleEdgeStyle(
    {
      stroke_color: target?.color ?? null,
      stroke_width: state.edgeWidth,
      opacity: edge.level === 1 ? 0.88 : 0.76,
      arrow_visible: true,
      label_visible: false
    },
    pixelRatio
  );
}

function orderEdgeStyle(edge, pixelRatio) {
  const band = currentIndex().bands.find((entry) => entry.id === edge.bandId);
  return scaleEdgeStyle(
    {
      stroke_color: band?.orderColor ?? "#64748b",
      stroke_width: Math.max(1, state.edgeWidth - 0.5),
      opacity: edge.level === 1 ? 0.46 : 0.42,
      arrow_visible: false,
      label_visible: false
    },
    pixelRatio
  );
}

function buildSpec() {
  const scenario = currentScenario();
  const index = currentIndex();
  const pixelRatio = state.pixelRatio;
  const positions = computeNodePositions(dom.canvas.width, dom.canvas.height);
  const offsetX = Number.isFinite(state.offsetX) ? Math.round(state.offsetX) : 0;
  const offsetY = Number.isFinite(state.offsetY) ? Math.round(state.offsetY) : 0;
  state.layoutSnapshot = positions;

  return {
    width: dom.canvas.width,
    height: dom.canvas.height,
    title: scenario.title,
    node_radius: scaleSize(state.nodeRadius, pixelRatio, 1),
    margin: scaleSize(34, pixelRatio, 1),
    offset_x: offsetX,
    offset_y: offsetY,
    pixel_ratio: pixelRatio,
    selected_node_id: isHiddenRootNodeId(state.selectedNodeId) ? null : state.selectedNodeId,
    layout_iterations: 1,
    show_arrows: true,
    show_labels: true,
    default_node_style: scaleNodeStyle(currentPreset().nodeStyle, pixelRatio),
    default_edge_style: scaleEdgeStyle(
      { ...currentPreset().edgeStyle, stroke_width: state.edgeWidth },
      pixelRatio
    ),
    selection_style: scaleSelectionStyle(currentPreset().selectionStyle, pixelRatio),
    nodes: index.nodes.map((node) => {
      const position = positions.get(node.id);
      const hiddenRoot = isHiddenRootNodeId(node.id);
      const style = hiddenRoot
        ? {
            ...(scaleNodeStyle(node.style, pixelRatio) ?? {}),
            radius: 1,
            opacity: 0,
            stroke_width: 0,
            label_visible: false
          }
        : scaleNodeStyle(node.style, pixelRatio);
      return {
        id: node.id,
        label: hiddenRoot ? "" : node.label,
        name: hiddenRoot ? null : node.name ?? null,
        color: node.color ?? null,
        x: position?.x ?? null,
        y: position?.y ?? null,
        shape: node.shape ?? "circle",
        label_inside: hiddenRoot ? true : node.labelInside ? true : undefined,
        style,
        media: node.media ?? null,
        properties: node.properties ?? {}
      };
    }),
    edges: [
      ...index.hierarchyEdges.map((edge) => ({
        source: edge.source,
        target: edge.target,
        label: null,
        color: null,
        weight: null,
        style: hierarchyEdgeStyle(edge, pixelRatio)
      })),
      ...index.orderEdges.map((edge) => ({
        source: edge.source,
        target: edge.target,
        label: null,
        color: null,
        weight: null,
        style: orderEdgeStyle(edge, pixelRatio)
      }))
    ]
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

function syncSession() {
  if (!supportsSessions()) {
    state.sessionHandle = null;
    state.sessionDirty = false;
    return;
  }

  normalizeSessionOffsets();
  destroySession();
  state.zoom = 1;
  state.sessionHandle = wasm.create_network_session("level-ordered-tree-canvas", buildSpec());
  state.sessionDirty = false;
}

function renderOrderBadges() {
  dom.badgeLayer.replaceChildren();

  if (!state.showOrderBadges) {
    dom.badgeLayer.hidden = true;
    return;
  }

  const rect = dom.canvas.getBoundingClientRect();
  if (rect.width === 0 || rect.height === 0) {
    dom.badgeLayer.hidden = true;
    return;
  }

  dom.badgeLayer.hidden = false;
  const scaleX = rect.width / dom.canvas.width;
  const scaleY = rect.height / dom.canvas.height;

  for (const node of currentIndex().orderedNodes) {
    const position = state.layoutSnapshot.get(node.id);
    if (!position) {
      continue;
    }

    const screenX = position.x * state.zoom + state.offsetX;
    const screenY = position.y * state.zoom + state.offsetY;
    const radius = scaleSize(node.style?.radius ?? state.nodeRadius, state.pixelRatio, 1) * state.zoom;
    const cssLeft = (screenX + radius * 0.72) * scaleX;
    const cssTop = (screenY - radius * 0.86) * scaleY;

    if (cssLeft < -28 || cssLeft > rect.width + 28 || cssTop < -28 || cssTop > rect.height + 28) {
      continue;
    }

    const badge = document.createElement("span");
    badge.className = `order-badge is-level-${node.level}`;
    badge.textContent = String(node.orderIndex);
    badge.style.left = `${cssLeft}px`;
    badge.style.top = `${cssTop}px`;
    dom.badgeLayer.append(badge);
  }
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
      wasm.render_network("level-ordered-tree-canvas", buildSpec());
      state.sessionDirty = false;
    }

    renderOrderBadges();

    if (state.statusKind !== "success") {
      setStatus(
        "success",
        "WASM ready.",
        "Drag to pan, scroll to zoom, and inspect order slots from the selected ordered hierarchy."
      );
    }
  } catch (error) {
    dom.badgeLayer.replaceChildren();
    console.error("[level-ordered-tree] render failed", error);
    setStatus("error", "Ordered tree render failed.", error?.message ?? String(error));
  }
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
  invalidateSession();
  return true;
}

function eventPoint(event) {
  const rect = dom.canvas.getBoundingClientRect();
  return {
    x: (event.clientX - rect.left) * (dom.canvas.width / rect.width),
    y: (event.clientY - rect.top) * (dom.canvas.height / rect.height)
  };
}

function resetView() {
  state.selectedNodeId = null;
  state.offsetX = 0;
  state.offsetY = 0;
  state.zoom = 1;
  state.sessionDirty = true;
  renderDetails();
  render();
}

function applyScenario(nextScenario) {
  const scenario = SCENARIOS[nextScenario];
  state.scenario = nextScenario;
  state.selectedNodeId = null;
  state.offsetX = 0;
  state.offsetY = 0;
  state.zoom = 1;
  state.nodeRadius = scenario.defaults.nodeRadius;
  state.edgeWidth = scenario.defaults.edgeWidth;
  state.showOrderBadges = scenario.defaults.showOrderBadges;
  state.sessionDirty = true;

  syncControls();
  renderMeta();
  renderDetails();
  render();
}

function attachControls() {
  for (const button of dom.scenarioButtons) {
    button.addEventListener("click", () => {
      applyScenario(button.dataset.scenario);
    });
  }

  for (const button of dom.presetButtons) {
    button.addEventListener("click", () => {
      state.preset = button.dataset.preset;
      invalidateSession();
      renderMeta();
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
    invalidateSession();
    render();
  });

  dom.edgeWidth.addEventListener("input", () => {
    state.edgeWidth = Number(dom.edgeWidth.value);
    dom.edgeWidthValue.textContent = `${state.edgeWidth}px`;
    invalidateSession();
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
    render();
  });

  dom.showOrderBadges.addEventListener("change", () => {
    state.showOrderBadges = dom.showOrderBadges.checked;
    renderMeta();
    renderDetails();
    renderOrderBadges();
  });

  dom.resetButton.addEventListener("click", resetView);
}

function setSelection(nodeId) {
  state.selectedNodeId = isHiddenRootNodeId(nodeId) ? null : nodeId;
  if (supportsSessions() && state.sessionHandle != null) {
    wasm.set_network_selection(
      state.sessionHandle,
      isHiddenRootNodeId(nodeId) ? undefined : nodeId ?? undefined
    );
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
      renderOrderBadges();
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
      console.error("[level-ordered-tree] pick failed", error);
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

        const nextZoom = Math.min(MAX_SCREEN_ZOOM, Math.max(MIN_SCREEN_ZOOM, state.zoom * factor));
        const ratio = nextZoom / state.zoom;
        state.offsetX = point.x - (point.x - state.offsetX) * ratio;
        state.offsetY = point.y - (point.y - state.offsetY) * ratio;
        state.zoom = nextZoom;

        wasm.zoom_network_session(state.sessionHandle, point.x, point.y, factor);
        wasm.render_network_session(state.sessionHandle);
        renderOrderBadges();
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
    console.error("[level-ordered-tree] wasm init failed", error);
    setStatus("error", "Unable to load WASM.", error?.message ?? String(error));
    return;
  }

  state.pixelRatio = Math.max(1, Math.min(3, Math.round(state.pixelRatio * 2) / 2));

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
      render();
    });
  });

  observer.observe(dom.canvas);
  window.addEventListener("beforeunload", destroySession, { once: true });
}

boot();
