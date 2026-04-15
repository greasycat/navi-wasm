import init, * as wasm from "../../pkg/navi_plot_wasm.js";

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
        console.warn(`[tree] image preload failed for ${key}`, error);
      }
    })
  );
}

const SCENARIOS = {
  stellar: {
    kicker: "Evolution branches from a dense molecular nursery",
    title: "Stellar evolution pathways",
    summary:
      "A compact lineage from fragmented cold gas into low-mass dwarfs, sun-like stars, and the short-lived massive branch that ends in compact remnants.",
    note:
      "This dataset is tuned to read like a clean observatory wall graphic. Branch colors stay category-driven, while the frontend controls expose node outlines, edge weight, and label placement.",
    facts: [
      { label: "Best for", value: "Showing stable branch depth without losing labels." },
      { label: "Primary tracers", value: "Dust lanes, infrared excess, ionized ejecta." },
      { label: "Curated lens", value: "A physical storyline, not a full stellar census." }
    ],
    legend: [
      { label: "Collapse and fragmentation", color: "#0f766e" },
      { label: "Sun-like evolution", color: "#d97706" },
      { label: "Massive stellar endpoints", color: "#b42318" }
    ],
    defaults: {
      nodeRadius: 18,
      levelGap: 104,
      siblingGap: 54,
      edgeWidth: 2,
      forceLabelInside: false
    },
    rootId: "cloud",
    nodes: [
      {
        id: "cloud",
        label: "Cloud",
        name: "Molecular cloud",
        color: "#0f766e",
        shape: "diamond",
        labelInside: true,
        style: { radius: 24, stroke_color: "#0b3d3a", stroke_width: 2 },
        media: { kind: "icon", icon: "galaxy", scale: 0.8, tint_color: "#f8fafc" },
        properties: { phase: "cold gas", scale: "80 pc filament", tracer: "CO and dust" }
      },
      {
        id: "cores",
        label: "Cores",
        name: "Fragmented dense cores",
        color: "#1d4ed8",
        shape: "square",
        labelInside: true,
        media: { kind: "icon", icon: "database", scale: 0.64, tint_color: "#eff6ff" },
        properties: { phase: "fragmentation", density: "10^4 cm^-3", view: "millimeter maps" }
      },
      {
        id: "low-track",
        label: "Low mass",
        name: "Low-mass channel",
        color: "#0f766e",
        shape: "square",
        labelInside: false,
        properties: { mass: "< 0.5 Msun", cadence: "slow", outcome: "long-lived dwarfs" }
      },
      {
        id: "solar-track",
        label: "Solar",
        name: "Solar-mass channel",
        color: "#d97706",
        shape: "square",
        labelInside: false,
        properties: { mass: "0.8 - 1.2 Msun", cadence: "moderate", outcome: "white dwarf" }
      },
      {
        id: "massive-track",
        label: "Massive",
        name: "Massive-star channel",
        color: "#b42318",
        shape: "square",
        labelInside: false,
        properties: { mass: "> 8 Msun", cadence: "fast", outcome: "compact remnant" }
      },
      {
        id: "low-proto",
        label: "Proto",
        name: "Low-mass protostar",
        color: "#14b8a6",
        shape: "circle",
        labelInside: true,
        properties: { phase: "Class I", tracer: "far infrared", envelope: "dust rich" }
      },
      {
        id: "brown-dwarf",
        label: "BD",
        name: "Brown dwarf",
        color: "#2dd4bf",
        shape: "triangle",
        labelInside: true,
        properties: { phase: "substellar", surface: "cool atmosphere", rarity: "common" }
      },
      {
        id: "m-dwarf",
        label: "M dwarf",
        name: "Active M dwarf",
        color: "#0f766e",
        shape: "circle",
        properties: { phase: "main sequence", activity: "flares", lifetime: "> 100 Gyr" }
      },
      {
        id: "red-dwarf",
        label: "Quiet red dwarf",
        name: "Quiescent red dwarf",
        color: "#0b5f58",
        shape: "circle",
        properties: { phase: "settled dwarf", activity: "reduced", planets: "temperate hosts" }
      },
      {
        id: "solar-proto",
        label: "Proto",
        name: "Solar-mass protostar",
        color: "#f59e0b",
        shape: "circle",
        labelInside: true,
        properties: { phase: "Class I", tracer: "infrared", accretion: "disk fed" }
      },
      {
        id: "t-tauri",
        label: "T Tauri",
        name: "T Tauri star",
        color: "#fbbf24",
        shape: "circle",
        properties: { phase: "pre-main sequence", tracer: "H alpha", variability: "high" }
      },
      {
        id: "solar-main",
        label: "Solar analog",
        name: "Sun-like main sequence star",
        color: "#f97316",
        shape: "circle",
        properties: { phase: "stable burning", duration: "10 Gyr", survey: "planet searches" }
      },
      {
        id: "red-giant",
        label: "Red giant",
        name: "Red giant branch",
        color: "#ea580c",
        shape: "circle",
        properties: { phase: "envelope expansion", radius: "100 Rsun", tracer: "near infrared" }
      },
      {
        id: "planetary-nebula",
        label: "Nebula",
        name: "Planetary nebula",
        color: "#fb923c",
        shape: "triangle",
        media: {
          kind: "image",
          image_key: "planetary-nebula",
          fit: "cover",
          scale: 0.86,
          fallback_icon: "camera"
        },
        properties: { phase: "shell ejection", visibility: "10 kyr", tracer: "[OIII]" }
      },
      {
        id: "white-dwarf",
        label: "WD",
        name: "White dwarf",
        color: "#94a3b8",
        shape: "diamond",
        labelInside: true,
        media: { kind: "icon", icon: "star", scale: 0.68, tint_color: "#ffffff" },
        properties: { phase: "compact remnant", mass: "0.6 Msun", cooling: "billions of years" }
      },
      {
        id: "massive-proto",
        label: "Proto",
        name: "Massive protostar",
        color: "#dc2626",
        shape: "circle",
        labelInside: true,
        style: { radius: 20 },
        properties: { phase: "embedded", tracer: "masers", feedback: "strong" }
      },
      {
        id: "blue-supergiant",
        label: "Blue SG",
        name: "Blue supergiant",
        color: "#ef4444",
        shape: "circle",
        labelInside: true,
        properties: { phase: "hot luminous", wind: "fast", temperature: "20 kK" }
      },
      {
        id: "red-supergiant",
        label: "Red SG",
        name: "Red supergiant",
        color: "#f97316",
        shape: "circle",
        labelInside: true,
        properties: { phase: "extended envelope", radius: "1000 Rsun", tracer: "infrared" }
      },
      {
        id: "core-collapse",
        label: "Collapse",
        name: "Core-collapse supernova",
        color: "#b42318",
        shape: "diamond",
        labelInside: true,
        style: { radius: 22, stroke_color: "#7f1d1d", stroke_width: 2 },
        media: { kind: "icon", icon: "alert", scale: 0.7, tint_color: "#fff7ed" },
        properties: { phase: "explosion", signal: "optical and neutrinos", duration: "weeks" }
      },
      {
        id: "neutron-star",
        label: "NS",
        name: "Neutron star",
        color: "#7c3aed",
        shape: "square",
        labelInside: true,
        properties: { phase: "compact remnant", density: "nuclear", signal: "X-ray and radio" }
      },
      {
        id: "black-hole",
        label: "BH",
        name: "Black hole",
        color: "#111827",
        shape: "square",
        labelInside: true,
        properties: { phase: "compact remnant", horizon: "present", signal: "accretion or GW" }
      }
    ],
    edges: [
      { source: "cloud", target: "cores", style: { stroke_color: "#0f766e", opacity: 0.82 } },
      { source: "cores", target: "low-track", style: { stroke_color: "#0f766e" } },
      { source: "cores", target: "solar-track", style: { stroke_color: "#d97706" } },
      { source: "cores", target: "massive-track", style: { stroke_color: "#b42318" } },
      { source: "low-track", target: "low-proto" },
      { source: "low-proto", target: "brown-dwarf", style: { stroke_color: "#14b8a6" } },
      { source: "low-proto", target: "m-dwarf", style: { stroke_color: "#0f766e" } },
      { source: "m-dwarf", target: "red-dwarf", style: { stroke_color: "#0b5f58" } },
      { source: "solar-track", target: "solar-proto" },
      { source: "solar-proto", target: "t-tauri" },
      { source: "t-tauri", target: "solar-main" },
      { source: "solar-main", target: "red-giant" },
      { source: "red-giant", target: "planetary-nebula", style: { stroke_color: "#fb923c" } },
      { source: "planetary-nebula", target: "white-dwarf", style: { stroke_color: "#94a3b8" } },
      { source: "massive-track", target: "massive-proto", style: { stroke_color: "#dc2626" } },
      { source: "massive-proto", target: "blue-supergiant", style: { stroke_color: "#ef4444" } },
      { source: "blue-supergiant", target: "red-supergiant", style: { stroke_color: "#f97316" } },
      { source: "red-supergiant", target: "core-collapse", style: { stroke_color: "#b42318", stroke_width: 3 } },
      { source: "core-collapse", target: "neutron-star", style: { stroke_color: "#7c3aed" } },
      { source: "core-collapse", target: "black-hole", style: { stroke_color: "#111827" } }
    ]
  },
  deepfield: {
    kicker: "Taxonomy for a mixed deep-field cutout",
    title: "Deep field taxonomy",
    summary:
      "A compact classification tree for the objects that dominate a long-exposure field: faint galaxies, active nuclei, foreground stars, and rare transient events.",
    note:
      "This branch set is useful when you want long labels and very different object classes on the same canvas. The quieter preset turns it into a catalog plate, while the stronger presets emphasize category color.",
    facts: [
      { label: "Best for", value: "Label density and mixed node shapes." },
      { label: "Primary tracers", value: "Photometric redshift, morphology, variability." },
      { label: "Curated lens", value: "Representative classes from one editorial field." }
    ],
    legend: [
      { label: "Galaxy populations", color: "#2563eb" },
      { label: "Active nuclei", color: "#b42318" },
      { label: "Foreground and stellar contaminants", color: "#0f766e" },
      { label: "Transient channels", color: "#d97706" }
    ],
    defaults: {
      nodeRadius: 17,
      levelGap: 98,
      siblingGap: 60,
      edgeWidth: 2,
      forceLabelInside: false
    },
    rootId: "deep-field",
    nodes: [
      {
        id: "deep-field",
        label: "Field",
        name: "Ultra-deep pointing",
        color: "#1f2937",
        shape: "diamond",
        labelInside: true,
        style: { radius: 24, stroke_color: "#0f172a", stroke_width: 2 },
        properties: { exposure: "160 hr stack", filter_set: "optical to near IR", aim: "taxonomy" }
      },
      {
        id: "galaxies",
        label: "Galaxies",
        name: "Galaxy branch",
        color: "#2563eb",
        shape: "square",
        labelInside: true,
        properties: { count: "dominant population", signal: "morphology and colors" }
      },
      {
        id: "quasars",
        label: "AGN",
        name: "Active nuclei branch",
        color: "#b42318",
        shape: "square",
        labelInside: true,
        properties: { count: "sparse but bright", signal: "broad lines and variability" }
      },
      {
        id: "stars",
        label: "Stars",
        name: "Foreground star branch",
        color: "#0f766e",
        shape: "square",
        labelInside: true,
        properties: { count: "foreground", signal: "proper colors and PSF" }
      },
      {
        id: "transients",
        label: "Transients",
        name: "Transient branch",
        color: "#d97706",
        shape: "square",
        labelInside: true,
        properties: { count: "rare", signal: "multi-epoch difference imaging" }
      },
      {
        id: "lyman-break",
        label: "LBG",
        name: "Lyman-break galaxy",
        color: "#3b82f6",
        shape: "circle",
        labelInside: true,
        properties: { redshift: "z ~ 6", signal: "dropout colors", morphology: "compact" }
      },
      {
        id: "barred-spiral",
        label: "Barred spiral",
        name: "Barred spiral",
        color: "#60a5fa",
        shape: "circle",
        properties: { redshift: "z ~ 0.6", signal: "resolved arms", morphology: "disk" }
      },
      {
        id: "dusty-merger",
        label: "Dusty merger",
        name: "Dust-obscured merger",
        color: "#1d4ed8",
        shape: "triangle",
        properties: { redshift: "z ~ 2", signal: "ir excess", morphology: "tidal tails" }
      },
      {
        id: "green-pea",
        label: "Green pea",
        name: "Compact starburst",
        color: "#38bdf8",
        shape: "triangle",
        properties: { redshift: "z ~ 0.3", signal: "[OIII] boost", morphology: "compact" }
      },
      {
        id: "broad-line",
        label: "Broad-line",
        name: "Broad-line quasar",
        color: "#ef4444",
        shape: "circle",
        properties: { redshift: "z ~ 2.1", signal: "broad emission", brightness: "high" }
      },
      {
        id: "obscured-agn",
        label: "Obscured AGN",
        name: "Obscured nucleus",
        color: "#dc2626",
        shape: "triangle",
        properties: { redshift: "z ~ 1.4", signal: "mid-IR excess", brightness: "moderate" }
      },
      {
        id: "lens-candidate",
        label: "Lens cand.",
        name: "Lensed quasar candidate",
        color: "#f87171",
        shape: "diamond",
        labelInside: true,
        properties: { redshift: "z ~ 2.7", signal: "multiple images", rarity: "rare" }
      },
      {
        id: "cool-dwarf",
        label: "Cool dwarf",
        name: "Cool dwarf interloper",
        color: "#14b8a6",
        shape: "circle",
        properties: { subtype: "late M", signal: "stellar colors", motion: "foreground" }
      },
      {
        id: "halo-giant",
        label: "Halo giant",
        name: "Halo giant",
        color: "#0f766e",
        shape: "triangle",
        properties: { subtype: "K giant", signal: "resolved PSF", motion: "foreground" }
      },
      {
        id: "wd-interloper",
        label: "WD",
        name: "White dwarf interloper",
        color: "#94a3b8",
        shape: "diamond",
        labelInside: true,
        properties: { subtype: "DA", signal: "blue colors", motion: "foreground" }
      },
      {
        id: "type-ia",
        label: "Type Ia",
        name: "Type Ia supernova",
        color: "#f59e0b",
        shape: "circle",
        properties: { redshift: "z ~ 0.8", signal: "standard candle", cadence: "days" }
      },
      {
        id: "kilonova",
        label: "Kilonova",
        name: "Kilonova candidate",
        color: "#d97706",
        shape: "diamond",
        labelInside: true,
        properties: { redshift: "local", signal: "fast red fade", rarity: "very rare" }
      },
      {
        id: "tidal-disruption",
        label: "TDE",
        name: "Tidal disruption event",
        color: "#fb923c",
        shape: "triangle",
        properties: { redshift: "z ~ 0.2", signal: "nuclear flare", cadence: "weeks" }
      },
      {
        id: "fast-blue",
        label: "Fast blue",
        name: "Fast blue optical transient",
        color: "#fdba74",
        shape: "triangle",
        properties: { redshift: "z ~ 0.5", signal: "blue rise", cadence: "hours" }
      }
    ],
    edges: [
      { source: "deep-field", target: "galaxies", style: { stroke_color: "#2563eb" } },
      { source: "deep-field", target: "quasars", style: { stroke_color: "#b42318" } },
      { source: "deep-field", target: "stars", style: { stroke_color: "#0f766e" } },
      { source: "deep-field", target: "transients", style: { stroke_color: "#d97706" } },
      { source: "galaxies", target: "lyman-break" },
      { source: "galaxies", target: "barred-spiral" },
      { source: "galaxies", target: "dusty-merger" },
      { source: "galaxies", target: "green-pea" },
      { source: "quasars", target: "broad-line" },
      { source: "quasars", target: "obscured-agn" },
      { source: "quasars", target: "lens-candidate", style: { stroke_width: 3 } },
      { source: "stars", target: "cool-dwarf" },
      { source: "stars", target: "halo-giant" },
      { source: "stars", target: "wd-interloper" },
      { source: "transients", target: "type-ia" },
      { source: "transients", target: "kilonova", style: { stroke_color: "#b45309", stroke_width: 3 } },
      { source: "transients", target: "tidal-disruption" },
      { source: "transients", target: "fast-blue" }
    ]
  },
  program: {
    kicker: "A mission-style branch set for a survey pipeline",
    title: "Survey program tree",
    summary:
      "A clean program hierarchy that tracks an imagined wide-field campaign from observing modes through calibration and public data products.",
    note:
      "This tree is less about astrophysics and more about structure. It gives frontend developers a practical graph with repeated branch widths, long product names, and a few highlighted milestones.",
    facts: [
      { label: "Best for", value: "Process trees, mission roadmaps, or program taxonomies." },
      { label: "Primary tracers", value: "Observing tier, calibration path, release packaging." },
      { label: "Curated lens", value: "Inspired by Roman-style survey planning." }
    ],
    legend: [
      { label: "Survey modes", color: "#0f766e" },
      { label: "Calibration work", color: "#2563eb" },
      { label: "Release products", color: "#b42318" }
    ],
    defaults: {
      nodeRadius: 17,
      levelGap: 94,
      siblingGap: 52,
      edgeWidth: 2,
      forceLabelInside: false
    },
    rootId: "roman-wide",
    nodes: [
      {
        id: "roman-wide",
        label: "Roman",
        name: "Wide-field survey program",
        color: "#0f172a",
        shape: "diamond",
        labelInside: true,
        style: { radius: 24, stroke_color: "#0f172a", stroke_width: 2 },
        properties: { mission: "Roman-like", cadence: "multi-season", status: "planning" }
      },
      {
        id: "high-latitude",
        label: "High latitude",
        name: "High-latitude survey",
        color: "#0f766e",
        shape: "square",
        labelInside: true,
        properties: { area: "2000 sq deg", science: "dark energy and galaxies" }
      },
      {
        id: "time-domain",
        label: "Time domain",
        name: "Time-domain survey",
        color: "#d97706",
        shape: "square",
        labelInside: true,
        properties: { area: "rolling fields", science: "supernovae and microlensing" }
      },
      {
        id: "calibration",
        label: "Calibration",
        name: "Calibration branch",
        color: "#2563eb",
        shape: "square",
        labelInside: true,
        properties: { area: "reference visits", science: "stability and zeropoints" }
      },
      {
        id: "products",
        label: "Products",
        name: "Public data products",
        color: "#b42318",
        shape: "square",
        labelInside: true,
        properties: { area: "archive", science: "community delivery" }
      },
      {
        id: "prism-field",
        label: "Prism field",
        name: "Slitless prism tier",
        color: "#14b8a6",
        shape: "circle",
        properties: { exposure: "deep", output: "low-res spectra", mode: "survey" }
      },
      {
        id: "weak-lensing",
        label: "Weak lensing",
        name: "Weak-lensing imaging tier",
        color: "#0f766e",
        shape: "circle",
        properties: { exposure: "broad filters", output: "shape catalog", mode: "survey" }
      },
      {
        id: "galaxy-redshift",
        label: "Redshift tiles",
        name: "Galaxy redshift tiles",
        color: "#2dd4bf",
        shape: "triangle",
        properties: { exposure: "repeat visits", output: "redshift map", mode: "survey" }
      },
      {
        id: "cadence-tier",
        label: "Cadence tier",
        name: "Cadence control tier",
        color: "#f59e0b",
        shape: "circle",
        properties: { exposure: "5 day spacing", output: "visit schedule", mode: "time domain" }
      },
      {
        id: "supernova-broker",
        label: "SN broker",
        name: "Supernova broker lane",
        color: "#d97706",
        shape: "circle",
        labelInside: true,
        properties: { exposure: "difference imaging", output: "alert packets", mode: "time domain" }
      },
      {
        id: "microlensing",
        label: "Microlensing",
        name: "Microlensing monitor",
        color: "#fb923c",
        shape: "triangle",
        properties: { exposure: "high cadence", output: "light curves", mode: "time domain" }
      },
      {
        id: "standard-stars",
        label: "Standards",
        name: "Standard-star ladder",
        color: "#3b82f6",
        shape: "circle",
        properties: { visit: "nightly", output: "zeropoints", mode: "calibration" }
      },
      {
        id: "flat-fields",
        label: "Flat fields",
        name: "Flat-field drift checks",
        color: "#60a5fa",
        shape: "circle",
        properties: { visit: "weekly", output: "response map", mode: "calibration" }
      },
      {
        id: "psf-monitor",
        label: "PSF monitor",
        name: "PSF stability monitor",
        color: "#1d4ed8",
        shape: "triangle",
        properties: { visit: "every orbit", output: "focus metrics", mode: "calibration" }
      },
      {
        id: "coadds",
        label: "Coadds",
        name: "Deep coadd mosaics",
        color: "#ef4444",
        shape: "circle",
        properties: { release: "seasonal", output: "stacked tiles", audience: "science teams" }
      },
      {
        id: "photoz",
        label: "Photo-z",
        name: "Photometric redshift tiles",
        color: "#b42318",
        shape: "circle",
        properties: { release: "seasonal", output: "probability grids", audience: "science teams" }
      },
      {
        id: "archive-relay",
        label: "Archive",
        name: "Archive relay",
        color: "#991b1b",
        shape: "triangle",
        properties: { release: "continuous", output: "queryable bundles", audience: "archive" }
      },
      {
        id: "public-release",
        label: "Release",
        name: "Public data release",
        color: "#7f1d1d",
        shape: "diamond",
        labelInside: true,
        style: { radius: 20 },
        properties: { release: "annual", output: "docs and tiles", audience: "community" }
      }
    ],
    edges: [
      { source: "roman-wide", target: "high-latitude", style: { stroke_color: "#0f766e" } },
      { source: "roman-wide", target: "time-domain", style: { stroke_color: "#d97706" } },
      { source: "roman-wide", target: "calibration", style: { stroke_color: "#2563eb" } },
      { source: "roman-wide", target: "products", style: { stroke_color: "#b42318" } },
      { source: "high-latitude", target: "prism-field" },
      { source: "high-latitude", target: "weak-lensing" },
      { source: "high-latitude", target: "galaxy-redshift" },
      { source: "time-domain", target: "cadence-tier" },
      { source: "time-domain", target: "supernova-broker", style: { stroke_width: 3 } },
      { source: "time-domain", target: "microlensing" },
      { source: "calibration", target: "standard-stars" },
      { source: "calibration", target: "flat-fields" },
      { source: "calibration", target: "psf-monitor" },
      { source: "products", target: "coadds", style: { stroke_color: "#ef4444" } },
      { source: "products", target: "photoz", style: { stroke_color: "#b42318" } },
      { source: "products", target: "archive-relay", style: { stroke_color: "#991b1b" } },
      { source: "archive-relay", target: "public-release", style: { stroke_color: "#7f1d1d", stroke_width: 3 } }
    ]
  }
};

const APPEARANCE_PRESETS = {
  atlas: {
    title: "Atlas",
    nodeStyle: { stroke_color: "#e5eef1", stroke_width: 2, opacity: 0.96 },
    edgeStyle: { stroke_color: "#8393a1", opacity: 0.78 },
    selectionStyle: { stroke_color: "#c2410c", stroke_width: 3, padding: 10, opacity: 0.96 }
  },
  signal: {
    title: "Signal",
    nodeStyle: { stroke_color: "#0f172a", stroke_width: 2, opacity: 0.98 },
    edgeStyle: { stroke_color: "#0f766e", opacity: 0.84 },
    selectionStyle: { stroke_color: "#b42318", stroke_width: 3, padding: 10, opacity: 0.96 }
  },
  quiet: {
    title: "Quiet",
    nodeStyle: { stroke_color: "#cbd5e1", stroke_width: 1, opacity: 0.9 },
    edgeStyle: { stroke_color: "#94a3b8", opacity: 0.58 },
    selectionStyle: { stroke_color: "#334155", stroke_width: 2, padding: 8, opacity: 0.9 }
  }
};

const state = {
  scenario: "stellar",
  preset: "atlas",
  pixelRatio: Math.min(window.devicePixelRatio || 1, 3),
  offsetX: 0,
  offsetY: 0,
  selectedNodeId: null,
  collapsedNodeIds: [],
  nodeRadius: 18,
  levelGap: 104,
  siblingGap: 54,
  edgeWidth: 2,
  forceLabelInside: false,
  sessionHandle: null,
  transitionFrame: null,
  sessionDirty: true,
  statusKind: "info"
};

const TREE_COLLAPSE_ANIMATION_MS = 220;

const dom = {
  canvas: document.getElementById("tree-canvas"),
  statusPanel: document.getElementById("status-panel"),
  scenarioButtons: Array.from(document.querySelectorAll("[data-scenario]")),
  presetButtons: Array.from(document.querySelectorAll("[data-preset]")),
  resetButton: document.getElementById("tree-reset"),
  scenarioSelect: document.getElementById("scenario-select"),
  title: document.getElementById("tree-title"),
  kicker: document.getElementById("tree-kicker"),
  summary: document.getElementById("tree-summary"),
  note: document.getElementById("tree-note"),
  facts: document.getElementById("tree-facts"),
  stats: document.getElementById("tree-stats"),
  legend: document.getElementById("tree-legend"),
  details: document.getElementById("tree-details"),
  nodeRadius: document.getElementById("node-radius"),
  nodeRadiusValue: document.getElementById("node-radius-value"),
  levelGap: document.getElementById("level-gap"),
  levelGapValue: document.getElementById("level-gap-value"),
  siblingGap: document.getElementById("sibling-gap"),
  siblingGapValue: document.getElementById("sibling-gap-value"),
  edgeWidth: document.getElementById("edge-width"),
  edgeWidthValue: document.getElementById("edge-width-value"),
  pixelRatio: document.getElementById("pixel-ratio"),
  pixelRatioValue: document.getElementById("pixel-ratio-value"),
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

function scaleNodeStyle(style, pixelRatio, zoom = 1) {
  if (!style) {
    return null;
  }

  const next = clone(style);
  if (next.radius != null) {
    next.radius = scaleSize(next.radius, pixelRatio * zoom, 1);
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
  return true;
}

function eventPoint(event) {
  const rect = dom.canvas.getBoundingClientRect();
  return {
    x: (event.clientX - rect.left) * (dom.canvas.width / rect.width),
    y: (event.clientY - rect.top) * (dom.canvas.height / rect.height)
  };
}

function buildAdjacency(scenario) {
  const children = new Map();
  const parent = new Map();

  for (const node of scenario.nodes) {
    children.set(node.id, []);
  }

  for (const edge of scenario.edges) {
    children.get(edge.source).push(edge.target);
    parent.set(edge.target, edge.source);
  }

  return { children, parent };
}

function treeStats(scenario) {
  const { children } = buildAdjacency(scenario);
  const rootChildren = children.get(scenario.rootId) ?? [];
  let leafCount = 0;
  let maxDepth = 0;
  let primarySplit = rootChildren.length;
  const queue = [{ id: scenario.rootId, depth: 0 }];

  while (queue.length) {
    const current = queue.shift();
    const nextChildren = children.get(current.id) ?? [];
    if (!nextChildren.length) {
      leafCount += 1;
    }
    if (primarySplit <= 1 && nextChildren.length > 1) {
      primarySplit = nextChildren.length;
    }
    maxDepth = Math.max(maxDepth, current.depth);
    for (const child of nextChildren) {
      queue.push({ id: child, depth: current.depth + 1 });
    }
  }

  return {
    nodeCount: scenario.nodes.length,
    leafCount,
    maxDepth,
    rootBranches: primarySplit
  };
}

function lineageFor(nodeId, scenario) {
  const { parent } = buildAdjacency(scenario);
  const nodesById = new Map(scenario.nodes.map((node) => [node.id, node]));
  const lineage = [];
  let cursor = nodeId;

  while (cursor) {
    const node = nodesById.get(cursor);
    lineage.unshift(node ? displayName(node) : cursor);
    cursor = parent.get(cursor) ?? null;
  }

  return lineage;
}

function countDescendants(nodeId, scenario) {
  const { children } = buildAdjacency(scenario);
  let count = 0;
  const queue = [...(children.get(nodeId) ?? [])];

  while (queue.length) {
    const current = queue.shift();
    count += 1;
    queue.push(...(children.get(current) ?? []));
  }

  return count;
}

function nodeHasChildren(nodeId, scenario) {
  const { children } = buildAdjacency(scenario);
  return (children.get(nodeId) ?? []).length > 0;
}

function isDescendantOf(ancestorId, nodeId, scenario) {
  if (!ancestorId || !nodeId || ancestorId === nodeId) {
    return false;
  }

  const { parent } = buildAdjacency(scenario);
  let cursor = parent.get(nodeId) ?? null;
  while (cursor) {
    if (cursor === ancestorId) {
      return true;
    }
    cursor = parent.get(cursor) ?? null;
  }

  return false;
}

function setCollapsedNodeState(nodeId, collapsed) {
  const next = new Set(state.collapsedNodeIds);
  if (collapsed) {
    next.add(nodeId);
  } else {
    next.delete(nodeId);
  }
  state.collapsedNodeIds = [...next];
}

function updateButtonState(buttons, key, attribute) {
  for (const button of buttons) {
    const active = button.dataset[attribute] === key;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-pressed", String(active));
  }
}

function renderScenarioMeta() {
  const scenario = currentScenario();
  const stats = treeStats(scenario);

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
      <span class="stat-label">Leaf endpoints</span>
      <span class="stat-value">${stats.leafCount}</span>
    </article>
    <article class="stat-tile">
      <span class="stat-label">Max depth</span>
      <span class="stat-value">${stats.maxDepth}</span>
    </article>
    <article class="stat-tile">
      <span class="stat-label">Primary split</span>
      <span class="stat-value" data-tone="warm">${stats.rootBranches} branches</span>
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
      "Select a node to inspect its lineage, children, and survey-facing metadata.";

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

  const { children } = buildAdjacency(scenario);
  const lineage = lineageFor(node.id, scenario).join(" -> ");
  const childCount = (children.get(node.id) ?? []).length;
  const descendantCount = countDescendants(node.id, scenario);
  const collapsed = state.collapsedNodeIds.includes(node.id);

  const stack = document.createElement("div");
  stack.className = "selection-stack";

  const heading = document.createElement("h4");
  heading.textContent = displayName(node);

  const meta = document.createElement("p");
  meta.className = "details-meta";
  meta.textContent =
    `ID ${node.id} - ${childCount} child branches - ${descendantCount} descendants - ${collapsed ? "collapsed" : "expanded"}`;

  const lineageBlock = document.createElement("div");
  lineageBlock.className = "detail-block";
  lineageBlock.innerHTML = `
    <span class="detail-key">Lineage</span>
    <span class="detail-value">${lineage}</span>
  `;

  const shapeBlock = document.createElement("div");
  shapeBlock.className = "detail-block";
  const mediaLabel = node.media?.kind === "image"
    ? `image (${node.media.image_key})`
    : node.media?.icon
      ? `icon (${node.media.icon})`
      : "none";
  shapeBlock.innerHTML = `
    <span class="detail-key">Render treatment</span>
    <span class="detail-value">${node.shape ?? "circle"} node, media ${mediaLabel}${state.forceLabelInside ? ", labels forced inside" : ""}</span>
  `;

  stack.append(heading, meta, lineageBlock, shapeBlock, renderProperties(node.properties));
  dom.details.replaceChildren(stack);
}

function syncControls() {
  dom.nodeRadius.value = String(state.nodeRadius);
  dom.nodeRadiusValue.textContent = `${state.nodeRadius}px`;

  dom.levelGap.value = String(state.levelGap);
  dom.levelGapValue.textContent = `${state.levelGap}px`;

  dom.siblingGap.value = String(state.siblingGap);
  dom.siblingGapValue.textContent = `${state.siblingGap}px`;

  dom.edgeWidth.value = String(state.edgeWidth);
  dom.edgeWidthValue.textContent = `${state.edgeWidth}px`;

  dom.pixelRatio.value = String(state.pixelRatio);
  dom.pixelRatioValue.textContent = `${state.pixelRatio}x`;

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
    root_id: scenario.rootId,
    node_radius: scaleSize(state.nodeRadius, pixelRatio, 1),
    level_gap: scaleSize(state.levelGap, pixelRatio, 1),
    sibling_gap: scaleSize(state.siblingGap, pixelRatio, 1),
    margin: scaleSize(48, pixelRatio, 1),
    offset_x: offsetX,
    offset_y: offsetY,
    pixel_ratio: pixelRatio,
    selected_node_id: state.selectedNodeId,
    collapsed_node_ids: state.collapsedNodeIds.slice(),
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
      shape: node.shape ?? "circle",
      label_inside: state.forceLabelInside ? true : Boolean(node.labelInside),
      style: scaleNodeStyle(node.style, pixelRatio),
      media: node.media ?? null,
      properties: node.properties ?? {}
    })),
    edges: scenario.edges.map((edge) => ({
      source: edge.source,
      target: edge.target,
      style: scaleEdgeStyle(edge.style, pixelRatio)
    }))
  };
}

function supportsSessions() {
  return typeof wasm.create_tree_session === "function";
}

function destroySession() {
  if (!supportsSessions() || state.sessionHandle == null) {
    state.sessionHandle = null;
    return;
  }

  try {
    wasm.destroy_tree_session(state.sessionHandle);
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
  state.sessionHandle = wasm.create_tree_session("tree-canvas", buildSpec());
  state.sessionDirty = false;
}

function cancelCollapseAnimation(renderFinal = false) {
  if (state.transitionFrame != null) {
    cancelAnimationFrame(state.transitionFrame);
    state.transitionFrame = null;
  }

  if (!renderFinal) {
    return;
  }

  if (supportsSessions() && state.sessionHandle != null && !state.sessionDirty) {
    wasm.render_tree_session(state.sessionHandle);
    return;
  }

  wasm.render_tree("tree-canvas", buildSpec());
}

function render() {
  cancelCollapseAnimation(false);
  syncCanvasSize();

  try {
    if (supportsSessions()) {
      if (state.sessionHandle == null || state.sessionDirty) {
        syncSession();
      }
      wasm.render_tree_session(state.sessionHandle);
    } else {
      wasm.render_tree("tree-canvas", buildSpec());
      state.sessionDirty = false;
    }
    if (state.statusKind !== "success") {
      setStatus(
        "success",
        "WASM ready.",
        "Drag to pan, scroll to zoom, click to inspect, and double-click a node to collapse its descendants."
      );
    }
  } catch (error) {
    console.error("[tree] render failed", error);
    setStatus("error", "Tree render failed.", error?.message ?? String(error));
  }
}

function applyScenario(nextScenario) {
  const scenario = SCENARIOS[nextScenario];
  state.scenario = nextScenario;
  state.selectedNodeId = null;
  state.collapsedNodeIds = [];
  state.offsetX = 0;
  state.offsetY = 0;
  state.nodeRadius = scenario.defaults.nodeRadius;
  state.levelGap = scenario.defaults.levelGap;
  state.siblingGap = scenario.defaults.siblingGap;
  state.edgeWidth = scenario.defaults.edgeWidth;
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

function setSelection(nodeId) {
  state.selectedNodeId = nodeId;
  if (supportsSessions() && state.sessionHandle != null) {
    wasm.set_tree_selection(state.sessionHandle, nodeId ?? undefined);
  }
}

function pickNodeAt(point) {
  if (supportsSessions() && state.sessionHandle != null && !state.sessionDirty) {
    return wasm.pick_tree_node_session(state.sessionHandle, point.x, point.y);
  }

  return wasm.pick_tree_node(buildSpec(), point.x, point.y);
}

function toggleCollapsedNode(nodeId) {
  let collapsed = false;

  if (supportsSessions() && state.sessionHandle != null && !state.sessionDirty) {
    collapsed = wasm.toggle_tree_node_collapsed_session(state.sessionHandle, nodeId);
  } else {
    if (!nodeHasChildren(nodeId, currentScenario())) {
      return false;
    }
    const current = new Set(state.collapsedNodeIds);
    collapsed = !current.has(nodeId);
  }

  setCollapsedNodeState(nodeId, collapsed);
  if (
    collapsed &&
    state.selectedNodeId &&
    state.selectedNodeId !== nodeId &&
    isDescendantOf(nodeId, state.selectedNodeId, currentScenario())
  ) {
    state.selectedNodeId = nodeId;
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

function animateCollapse(nodeId) {
  if (!nodeHasChildren(nodeId, currentScenario())) {
    return;
  }

  cancelCollapseAnimation(false);
  toggleCollapsedNode(nodeId);
  renderDetails();

  if (
    !supportsSessions() ||
    state.sessionHandle == null ||
    state.sessionDirty ||
    typeof wasm.render_tree_session_transition !== "function"
  ) {
    render();
    return;
  }

  const start = performance.now();
  const step = (now) => {
    const progress = Math.min(1, (now - start) / TREE_COLLAPSE_ANIMATION_MS);
    wasm.render_tree_session_transition(
      state.sessionHandle,
      easeTreeTransition(progress)
    );

    if (progress < 1) {
      state.transitionFrame = requestAnimationFrame(step);
      return;
    }

    state.transitionFrame = null;
    render();
  };

  state.transitionFrame = requestAnimationFrame(step);
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

  dom.levelGap.addEventListener("input", () => {
    state.levelGap = Number(dom.levelGap.value);
    dom.levelGapValue.textContent = `${state.levelGap}px`;
    state.sessionDirty = true;
    render();
  });

  dom.siblingGap.addEventListener("input", () => {
    state.siblingGap = Number(dom.siblingGap.value);
    dom.siblingGapValue.textContent = `${state.siblingGap}px`;
    state.sessionDirty = true;
    render();
  });

  dom.edgeWidth.addEventListener("input", () => {
    state.edgeWidth = Number(dom.edgeWidth.value);
    dom.edgeWidthValue.textContent = `${state.edgeWidth}px`;
    state.sessionDirty = true;
    render();
  });

  dom.pixelRatio.addEventListener("input", () => {
    state.pixelRatio = Number(dom.pixelRatio.value);
    dom.pixelRatioValue.textContent = `${state.pixelRatio}x`;
    syncCanvasSize(true);
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

function attachCanvasInteractions() {
  const drag = {
    active: false,
    moved: false,
    start: { x: 0, y: 0 },
    last: { x: 0, y: 0 }
  };

  dom.canvas.addEventListener("pointerdown", (event) => {
    cancelCollapseAnimation(true);
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
    const dx = Math.round(point.x - drag.last.x);
    const dy = Math.round(point.y - drag.last.y);

    if (dx === 0 && dy === 0) {
      return;
    }

    drag.last = point;
    drag.moved =
      drag.moved || Math.hypot(point.x - drag.start.x, point.y - drag.start.y) > 4;
    state.offsetX += dx;
    state.offsetY += dy;
    if (supportsSessions() && state.sessionHandle != null && !state.sessionDirty) {
      wasm.pan_tree_session(state.sessionHandle, dx, dy);
      wasm.render_tree_session(state.sessionHandle);
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
      const hit = pickNodeAt(point);
      setSelection(hit?.node_id ?? null);
      renderDetails();
      render();
    } catch (error) {
      console.error("[tree] pick failed", error);
    }
  }

  dom.canvas.addEventListener("pointerup", finishPointer);
  dom.canvas.addEventListener("pointercancel", finishPointer);
  dom.canvas.addEventListener("dblclick", (event) => {
    event.preventDefault();

    try {
      const point = eventPoint(event);
      const hit = pickNodeAt(point);
      if (!hit?.node_id) {
        return;
      }

      animateCollapse(hit.node_id);
    } catch (error) {
      console.error("[tree] collapse failed", error);
    }
  });

  dom.canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      cancelCollapseAnimation(true);

      const point = eventPoint(event);
      const factor = event.deltaY < 0 ? 1.1 : 1 / 1.1;
      if (supportsSessions() && state.sessionHandle != null && !state.sessionDirty) {
        wasm.zoom_tree_session(state.sessionHandle, point.x, point.y, factor);
        wasm.render_tree_session(state.sessionHandle);
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
    console.error("[tree] wasm init failed", error);
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
