import init, * as wasm from "../../pkg/navi_plot_wasm.js";

// ─── Example trees ───────────────────────────────────────────────────────────

const EXAMPLES = {
  org: {
    title: "Engineering org chart",
    root_id: "cto",
    nodes: [
      // C-suite
      { id: "cto",         label: "CTO",        name: "Chief Technology Officer",  shape: "diamond", color: "#0f172a", properties: { level: "C-suite", reports: "4 VPs" } },
      // VPs — 4 direct reports
      { id: "vp-eng",      label: "VP Eng",     name: "VP Engineering",            shape: "square",  color: "#2563eb", properties: { level: "VP",     reports: "3 leads" } },
      { id: "vp-prod",     label: "VP Prod",    name: "VP Product",                shape: "square",  color: "#7c3aed", properties: { level: "VP",     reports: "3 ICs"   } },
      { id: "vp-data",     label: "VP Data",    name: "VP Data & ML",              shape: "square",  color: "#0891b2", properties: { level: "VP",     reports: "2 leads" } },
      { id: "vp-sec",      label: "VP Sec",     name: "VP Security",               shape: "square",  color: "#be123c", properties: { level: "VP",     reports: "2 leads" } },
      // Eng leads — 3 under VP Eng
      { id: "eng-lead",    label: "Eng Lead",   name: "Engineering Lead",          shape: "square",  color: "#1d4ed8", properties: { level: "Lead",   team: "Platform"     } },
      { id: "devops-lead", label: "DevOps",     name: "DevOps Lead",               shape: "square",  color: "#1d4ed8", properties: { level: "Lead",   team: "Reliability"  } },
      { id: "mobile-lead", label: "Mobile",     name: "Mobile Lead",               shape: "square",  color: "#1d4ed8", properties: { level: "Lead",   team: "iOS & Android"} },
      // Data leads — 2 under VP Data
      { id: "data-lead",   label: "Data Lead",  name: "Data Engineering Lead",     shape: "square",  color: "#0e7490", properties: { level: "Lead",   team: "Data Platform"} },
      { id: "ml-lead",     label: "ML Lead",    name: "ML Platform Lead",          shape: "square",  color: "#0e7490", properties: { level: "Lead",   team: "ML Platform"  } },
      // Sec leads — 2 under VP Sec
      { id: "appsec-lead", label: "AppSec",     name: "Application Security Lead", shape: "square",  color: "#9f1239", properties: { level: "Lead",   team: "AppSec"       } },
      { id: "infrasec",    label: "InfraSec",   name: "Infrastructure Security",   shape: "square",  color: "#9f1239", properties: { level: "Lead",   team: "InfraSec"     } },
      // IC leaves — Platform (3 SWEs)
      { id: "swe1",        label: "SWE",        name: "Software Engineer I",       shape: "circle",  color: "#60a5fa", properties: { level: "IC4",    lang: "Rust / Go"   } },
      { id: "swe2",        label: "SWE",        name: "Software Engineer II",      shape: "circle",  color: "#60a5fa", properties: { level: "IC3",    lang: "TypeScript"  } },
      { id: "swe3",        label: "SWE",        name: "Software Engineer III",     shape: "circle",  color: "#60a5fa", properties: { level: "IC5",    lang: "Rust"        } },
      // DevOps (2 SREs)
      { id: "sre1",        label: "SRE",        name: "Site Reliability Eng I",    shape: "circle",  color: "#93c5fd", properties: { level: "IC4",    focus: "Infrastructure"} },
      { id: "sre2",        label: "SRE",        name: "Site Reliability Eng II",   shape: "circle",  color: "#93c5fd", properties: { level: "IC3",    focus: "Observability" } },
      // Mobile (3 engineers)
      { id: "ios",         label: "iOS",        name: "iOS Engineer",              shape: "circle",  color: "#bfdbfe", properties: { level: "IC4",    lang: "Swift"       } },
      { id: "android",     label: "Android",    name: "Android Engineer",          shape: "circle",  color: "#bfdbfe", properties: { level: "IC4",    lang: "Kotlin"      } },
      { id: "rn",          label: "React-N",    name: "React Native Engineer",     shape: "circle",  color: "#bfdbfe", properties: { level: "IC3",    lang: "TypeScript"  } },
      // Product ICs (3 under VP Prod)
      { id: "pm1",         label: "PM",         name: "Product Manager — Core",    shape: "circle",  color: "#a78bfa", properties: { level: "IC4",    area: "Core"        } },
      { id: "pm2",         label: "PM",         name: "Product Manager — Growth",  shape: "circle",  color: "#a78bfa", properties: { level: "IC4",    area: "Growth"      } },
      { id: "designer",    label: "Design",     name: "UX Designer",               shape: "triangle",color: "#c4b5fd", properties: { level: "IC3",    tools: "Figma"      } },
      // Data (2 under data-lead)
      { id: "ds1",         label: "DS",         name: "Data Scientist I",          shape: "triangle",color: "#67e8f9", properties: { level: "IC4",    focus: "Experimentation"} },
      { id: "ds2",         label: "DS",         name: "Data Scientist II",         shape: "triangle",color: "#67e8f9", properties: { level: "IC3",    focus: "Analytics"     } },
      // ML (2 under ml-lead)
      { id: "mle1",        label: "MLE",        name: "ML Engineer — Ranking",     shape: "triangle",color: "#a5f3fc", properties: { level: "IC5",    focus: "Ranking models"} },
      { id: "mle2",        label: "MLE",        name: "ML Engineer — NLP",         shape: "triangle",color: "#a5f3fc", properties: { level: "IC4",    focus: "NLP"           } },
      // Sec ICs
      { id: "pen",         label: "PenTest",    name: "Penetration Tester",        shape: "triangle",color: "#fda4af", properties: { level: "IC4",    cert: "OSCP"           } },
      { id: "seceng",      label: "SecEng",     name: "Security Engineer",         shape: "triangle",color: "#fda4af", properties: { level: "IC4",    focus: "IAM"           } },
    ],
    edges: [
      { source: "cto",         target: "vp-eng"      },
      { source: "cto",         target: "vp-prod"     },
      { source: "cto",         target: "vp-data"     },
      { source: "cto",         target: "vp-sec"      },
      { source: "vp-eng",      target: "eng-lead"    },
      { source: "vp-eng",      target: "devops-lead" },
      { source: "vp-eng",      target: "mobile-lead" },
      { source: "vp-prod",     target: "pm1"         },
      { source: "vp-prod",     target: "pm2"         },
      { source: "vp-prod",     target: "designer"    },
      { source: "vp-data",     target: "data-lead"   },
      { source: "vp-data",     target: "ml-lead"     },
      { source: "vp-sec",      target: "appsec-lead" },
      { source: "vp-sec",      target: "infrasec"    },
      { source: "eng-lead",    target: "swe1"        },
      { source: "eng-lead",    target: "swe2"        },
      { source: "eng-lead",    target: "swe3"        },
      { source: "devops-lead", target: "sre1"        },
      { source: "devops-lead", target: "sre2"        },
      { source: "mobile-lead", target: "ios"         },
      { source: "mobile-lead", target: "android"     },
      { source: "mobile-lead", target: "rn"          },
      { source: "data-lead",   target: "ds1"         },
      { source: "data-lead",   target: "ds2"         },
      { source: "ml-lead",     target: "mle1"        },
      { source: "ml-lead",     target: "mle2"        },
      { source: "appsec-lead", target: "pen"         },
      { source: "appsec-lead", target: "seceng"      },
    ],
  },

  fs: {
    title: "File system snapshot",
    root_id: "root",
    nodes: [
      // Root dirs — 5 children
      { id: "root",     label: "/",           shape: "square",   color: "#1e293b", properties: { type: "dir",     perms: "rwxr-xr-x" } },
      { id: "home",     label: "home/",       shape: "square",   color: "#334155", properties: { type: "dir" } },
      { id: "etc",      label: "etc/",        shape: "square",   color: "#334155", properties: { type: "dir" } },
      { id: "usr",      label: "usr/",        shape: "square",   color: "#334155", properties: { type: "dir" } },
      { id: "var",      label: "var/",        shape: "square",   color: "#334155", properties: { type: "dir" } },
      { id: "tmp",      label: "tmp/",        shape: "square",   color: "#475569", properties: { type: "dir",     perms: "rwxrwxrwx" } },
      // home — 3 children
      { id: "bashrc",   label: ".bashrc",     shape: "circle",   color: "#64748b", properties: { type: "file",    size: "4.2 KB"  } },
      { id: "ssh-dir",  label: ".ssh/",       shape: "square",   color: "#475569", properties: { type: "dir" } },
      { id: "projects", label: "projects/",   shape: "square",   color: "#475569", properties: { type: "dir" } },
      // .ssh — 2 files
      { id: "id-rsa",   label: "id_rsa",      shape: "circle",   color: "#f43f5e", properties: { type: "file",    size: "3.4 KB",  perms: "rw-------" } },
      { id: "known",    label: "known_hosts", shape: "circle",   color: "#94a3b8", properties: { type: "file",    size: "8.1 KB"  } },
      // projects — 3 dirs
      { id: "app",      label: "app/",        shape: "square",   color: "#475569", properties: { type: "dir" } },
      { id: "data-dir", label: "data/",       shape: "square",   color: "#475569", properties: { type: "dir" } },
      { id: "scripts",  label: "scripts/",    shape: "square",   color: "#475569", properties: { type: "dir" } },
      // app — 3 files
      { id: "main-rs",  label: "main.rs",     shape: "circle",   color: "#f97316", properties: { type: "file",    size: "12.4 KB", lang: "Rust" } },
      { id: "cargo",    label: "Cargo.toml",  shape: "circle",   color: "#fb923c", properties: { type: "file",    size: "1.1 KB"  } },
      { id: "readme",   label: "README.md",   shape: "circle",   color: "#94a3b8", properties: { type: "file",    size: "2.8 KB"  } },
      // data — 2 files
      { id: "csv",      label: "input.csv",   shape: "circle",   color: "#22c55e", properties: { type: "file",    size: "2.1 MB"  } },
      { id: "schema",   label: "schema.sql",  shape: "circle",   color: "#4ade80", properties: { type: "file",    size: "14 KB"   } },
      // scripts — 2 files
      { id: "deploy",   label: "deploy.sh",   shape: "circle",   color: "#fbbf24", properties: { type: "file",    size: "3.6 KB",  lang: "Bash" } },
      { id: "lint",     label: "lint.sh",     shape: "circle",   color: "#fbbf24", properties: { type: "file",    size: "900 B",   lang: "Bash" } },
      // etc — 3 files
      { id: "hosts",    label: "hosts",       shape: "circle",   color: "#94a3b8", properties: { type: "file",    size: "512 B"   } },
      { id: "fstab",    label: "fstab",       shape: "circle",   color: "#94a3b8", properties: { type: "file",    size: "1.2 KB"  } },
      { id: "sudoers",  label: "sudoers",     shape: "circle",   color: "#f87171", properties: { type: "file",    perms: "r--r-----" } },
      // usr — 3 children
      { id: "bin",      label: "bin/",        shape: "square",   color: "#475569", properties: { type: "dir" } },
      { id: "lib",      label: "lib/",        shape: "square",   color: "#475569", properties: { type: "dir" } },
      { id: "share",    label: "share/",      shape: "square",   color: "#475569", properties: { type: "dir" } },
      // bin — 3 binaries
      { id: "python3",  label: "python3",     shape: "triangle", color: "#fbbf24", properties: { type: "binary",  version: "3.12.4" } },
      { id: "git",      label: "git",         shape: "triangle", color: "#f97316", properties: { type: "binary",  version: "2.44.0" } },
      { id: "curl",     label: "curl",        shape: "triangle", color: "#94a3b8", properties: { type: "binary",  version: "8.7.1"  } },
      // lib — 2 libs
      { id: "libssl",   label: "libssl.so",   shape: "triangle", color: "#94a3b8", properties: { type: "library", version: "3.0.2"  } },
      { id: "libz",     label: "libz.so",     shape: "triangle", color: "#cbd5e1", properties: { type: "library", version: "1.3.1"  } },
      // var — 2 children
      { id: "log",      label: "log/",        shape: "square",   color: "#475569", properties: { type: "dir" } },
      { id: "spool",    label: "spool/",      shape: "square",   color: "#475569", properties: { type: "dir" } },
      // log — 3 log files
      { id: "syslog",   label: "syslog",      shape: "circle",   color: "#64748b", properties: { type: "file",    size: "48 MB"   } },
      { id: "auth-log", label: "auth.log",    shape: "circle",   color: "#64748b", properties: { type: "file",    size: "12 MB"   } },
      { id: "kern-log", label: "kern.log",    shape: "circle",   color: "#64748b", properties: { type: "file",    size: "6.2 MB"  } },
    ],
    edges: [
      { source: "root",     target: "home"     },
      { source: "root",     target: "etc"      },
      { source: "root",     target: "usr"      },
      { source: "root",     target: "var"      },
      { source: "root",     target: "tmp"      },
      { source: "home",     target: "bashrc"   },
      { source: "home",     target: "ssh-dir"  },
      { source: "home",     target: "projects" },
      { source: "ssh-dir",  target: "id-rsa"   },
      { source: "ssh-dir",  target: "known"    },
      { source: "projects", target: "app"      },
      { source: "projects", target: "data-dir" },
      { source: "projects", target: "scripts"  },
      { source: "app",      target: "main-rs"  },
      { source: "app",      target: "cargo"    },
      { source: "app",      target: "readme"   },
      { source: "data-dir", target: "csv"      },
      { source: "data-dir", target: "schema"   },
      { source: "scripts",  target: "deploy"   },
      { source: "scripts",  target: "lint"     },
      { source: "etc",      target: "hosts"    },
      { source: "etc",      target: "fstab"    },
      { source: "etc",      target: "sudoers"  },
      { source: "usr",      target: "bin"      },
      { source: "usr",      target: "lib"      },
      { source: "usr",      target: "share"    },
      { source: "bin",      target: "python3"  },
      { source: "bin",      target: "git"      },
      { source: "bin",      target: "curl"     },
      { source: "lib",      target: "libssl"   },
      { source: "lib",      target: "libz"     },
      { source: "var",      target: "log"      },
      { source: "var",      target: "spool"    },
      { source: "log",      target: "syslog"   },
      { source: "log",      target: "auth-log" },
      { source: "log",      target: "kern-log" },
    ],
  },

  deps: {
    title: "Package dependency tree",
    root_id: "app",
    nodes: [
      // Root — 5 direct deps
      { id: "app",          label: "my-app",      name: "my-app v1.0.0",           shape: "diamond",  color: "#0f766e", properties: { version: "1.0.0",   type: "application"    } },
      { id: "react",        label: "react",        name: "react",                   shape: "square",   color: "#0ea5e9", properties: { version: "18.2.0",  weekly: "21M"          } },
      { id: "lodash",       label: "lodash",       name: "lodash",                  shape: "square",   color: "#8b5cf6", properties: { version: "4.17.21", weekly: "48M"          } },
      { id: "webpack",      label: "webpack",      name: "webpack",                 shape: "square",   color: "#f59e0b", properties: { version: "5.88.0",  weekly: "18M"          } },
      { id: "typescript",   label: "typescript",   name: "typescript",              shape: "square",   color: "#3b82f6", properties: { version: "5.1.6",   weekly: "45M"          } },
      { id: "axios",        label: "axios",        name: "axios",                   shape: "square",   color: "#10b981", properties: { version: "1.6.0",   weekly: "42M"          } },
      // react — 3 children
      { id: "react-dom",    label: "react-dom",    name: "react-dom",               shape: "circle",   color: "#38bdf8", properties: { version: "18.2.0",  peer: "react"          } },
      { id: "scheduler",    label: "scheduler",    name: "scheduler",               shape: "circle",   color: "#7dd3fc", properties: { version: "0.23.0",  internal: "true"       } },
      { id: "prop-types",   label: "prop-types",   name: "prop-types",              shape: "circle",   color: "#bae6fd", properties: { version: "15.8.1"                         } },
      // lodash — 3 children
      { id: "lodash-fp",    label: "lodash-fp",    name: "lodash-fp",               shape: "circle",   color: "#a78bfa", properties: { version: "0.10.4"                         } },
      { id: "lodash-chunk", label: "chunk",        name: "lodash.chunk",            shape: "circle",   color: "#c4b5fd", properties: { version: "4.2.0"                          } },
      { id: "lodash-merge", label: "merge",        name: "lodash.merge",            shape: "circle",   color: "#c4b5fd", properties: { version: "4.6.2"                          } },
      // webpack — 4 children
      { id: "babel-loader", label: "babel-ldr",    name: "babel-loader",            shape: "triangle", color: "#fcd34d", properties: { version: "9.1.3",   devDep: "true"        } },
      { id: "css-loader",   label: "css-ldr",      name: "css-loader",              shape: "triangle", color: "#fde68a", properties: { version: "6.8.1",   devDep: "true"        } },
      { id: "ts-loader",    label: "ts-ldr",       name: "ts-loader",               shape: "triangle", color: "#fef08a", properties: { version: "9.4.4",   devDep: "true"        } },
      { id: "file-loader",  label: "file-ldr",     name: "file-loader",             shape: "triangle", color: "#fef9c3", properties: { version: "6.2.0",   devDep: "true"        } },
      // typescript — 3 children
      { id: "tslib",        label: "tslib",        name: "tslib",                   shape: "circle",   color: "#93c5fd", properties: { version: "2.6.0",   size: "12 KB"         } },
      { id: "ts-node",      label: "ts-node",      name: "ts-node",                 shape: "circle",   color: "#bfdbfe", properties: { version: "10.9.1",  devDep: "true"        } },
      { id: "ts-jest",      label: "ts-jest",      name: "ts-jest",                 shape: "circle",   color: "#bfdbfe", properties: { version: "29.1.0",  devDep: "true"        } },
      // axios — 3 children
      { id: "follow-redir", label: "follow-redir", name: "follow-redirects",        shape: "circle",   color: "#6ee7b7", properties: { version: "1.15.3"                         } },
      { id: "form-data",    label: "form-data",    name: "form-data",               shape: "circle",   color: "#6ee7b7", properties: { version: "4.0.0"                          } },
      { id: "proxy-agent",  label: "proxy-agent",  name: "https-proxy-agent",       shape: "circle",   color: "#6ee7b7", properties: { version: "7.0.2"                          } },
    ],
    edges: [
      { source: "app",        target: "react"        },
      { source: "app",        target: "lodash"       },
      { source: "app",        target: "webpack"      },
      { source: "app",        target: "typescript"   },
      { source: "app",        target: "axios"        },
      { source: "react",      target: "react-dom"    },
      { source: "react",      target: "scheduler"    },
      { source: "react",      target: "prop-types"   },
      { source: "lodash",     target: "lodash-fp"    },
      { source: "lodash",     target: "lodash-chunk" },
      { source: "lodash",     target: "lodash-merge" },
      { source: "webpack",    target: "babel-loader" },
      { source: "webpack",    target: "css-loader"   },
      { source: "webpack",    target: "ts-loader"    },
      { source: "webpack",    target: "file-loader"  },
      { source: "typescript", target: "tslib"        },
      { source: "typescript", target: "ts-node"      },
      { source: "typescript", target: "ts-jest"      },
      { source: "axios",      target: "follow-redir" },
      { source: "axios",      target: "form-data"    },
      { source: "axios",      target: "proxy-agent"  },
    ],
  },
};

// Default layout params per example
const DEFAULT_PARAMS = {
  org:  { nodeRadius: 18, levelGap: 90, siblingGap: 44 },
  fs:   { nodeRadius: 16, levelGap: 80, siblingGap: 40 },
  deps: { nodeRadius: 18, levelGap: 88, siblingGap: 44 },
};

// ─── State ───────────────────────────────────────────────────────────────────

const state = {
  example:        "org",
  offsetX:        0,
  offsetY:        0,
  zoom:           1.0,
  pixelRatio:     Math.min(window.devicePixelRatio || 1, 4),
  selectedNodeId: null,
  shapeOverride:  null,   // null = use per-node shape from example data
  labelInside:    false,
};

// ─── DOM refs ────────────────────────────────────────────────────────────────

const canvas          = document.getElementById("tree-canvas");
const statusPanel     = document.getElementById("status-panel");
const detailsPanel    = document.getElementById("tree-details");
const exampleSelect   = document.getElementById("example-select");
const radiusInput     = document.getElementById("node-radius");
const radiusVal       = document.getElementById("node-radius-val");
const levelGapInput   = document.getElementById("level-gap");
const levelGapVal     = document.getElementById("level-gap-val");
const sibGapInput     = document.getElementById("sibling-gap");
const sibGapVal       = document.getElementById("sibling-gap-val");
const shapeBtns       = document.querySelectorAll(".shape-btn");
const labelInsideCheck= document.getElementById("label-inside");
const resetBtn        = document.getElementById("tree-reset");
const prInput         = document.getElementById("pixel-ratio");
const prVal           = document.getElementById("pixel-ratio-val");

// ─── Canvas coordinate helpers ───────────────────────────────────────────────

function toCanvasPt(e) {
  const rect = canvas.getBoundingClientRect();
  return {
    x: (e.clientX - rect.left) * (canvas.width  / rect.width),
    y: (e.clientY - rect.top)  * (canvas.height / rect.height),
  };
}

// ─── Canvas setup ────────────────────────────────────────────────────────────

// Lock canvas to its current CSS display size at pixel_ratio resolution.
// Must be called after layout is settled (post-init).
function setupCanvas() {
  const pr   = state.pixelRatio;
  const rect = canvas.getBoundingClientRect();
  const cssW = Math.round(rect.width);
  const cssH = Math.round(rect.height) || Math.round(cssW * 540 / 820);
  canvas.style.width  = cssW + "px";
  canvas.style.height = cssH + "px";
  canvas.width  = Math.round(cssW * pr);
  canvas.height = Math.round(cssH * pr);
}

// ─── Spec builder ────────────────────────────────────────────────────────────

function buildSpec() {
  const pr = state.pixelRatio;
  const ex = EXAMPLES[state.example];
  return {
    width:        canvas.width,
    height:       canvas.height,
    title:        ex.title,
    root_id:      ex.root_id,
    node_radius:  Math.round(parseInt(radiusInput.value)   * pr * state.zoom),
    level_gap:    Math.round(parseInt(levelGapInput.value)  * pr * state.zoom),
    sibling_gap:  Math.round(parseInt(sibGapInput.value)    * pr * state.zoom),
    margin:       Math.round(28 * pr),
    offset_x:     state.offsetX,
    offset_y:     state.offsetY,
    pixel_ratio:  pr,
    selected_node_id: state.selectedNodeId,
    nodes: ex.nodes.map(n => ({
      id:           n.id,
      label:        n.label,
      name:         n.name  ?? null,
      color:        n.color ?? null,
      shape:        state.shapeOverride ?? n.shape ?? "circle",
      label_inside: state.labelInside,
      properties:   n.properties ?? {},
    })),
    edges: ex.edges,
  };
}

// ─── Render ──────────────────────────────────────────────────────────────────

function render() {
  try {
    wasm.render_tree("tree-canvas", buildSpec());
  } catch (e) {
    console.error("[tree] render error:", e.message ?? e);
  }
}

// ─── Details panel ───────────────────────────────────────────────────────────

function renderDetails() {
  if (!state.selectedNodeId) {
    detailsPanel.replaceChildren(
      Object.assign(document.createElement("p"), {
        className:   "details-empty-copy",
        textContent: "Click a node to inspect it.",
      })
    );
    return;
  }

  const node = EXAMPLES[state.example].nodes.find(n => n.id === state.selectedNodeId);
  if (!node) return;

  const h3 = Object.assign(document.createElement("h3"), {
    textContent: node.name ?? node.label,
  });

  const effectiveShape = state.shapeOverride ?? node.shape ?? "circle";
  const meta = Object.assign(document.createElement("p"), {
    className:   "details-meta",
    textContent: `id: ${node.id}  ·  shape: ${effectiveShape}`,
  });

  const colorRow = document.createElement("div");
  colorRow.className = "color-row";
  const swatch = Object.assign(document.createElement("span"), { className: "color-swatch" });
  swatch.style.background = node.color ?? "#888";
  const colorCode = Object.assign(document.createElement("code"), {
    textContent: node.color ?? "auto",
  });
  colorRow.append(swatch, colorCode);

  const dl = document.createElement("dl");
  dl.className = "property-list";
  for (const [k, v] of Object.entries(node.properties ?? {})) {
    const row = document.createElement("div");
    row.className = "property-row";
    row.append(
      Object.assign(document.createElement("dt"), { textContent: k }),
      Object.assign(document.createElement("dd"), { textContent: v })
    );
    dl.appendChild(row);
  }

  detailsPanel.replaceChildren(h3, meta, colorRow, dl);
}

// ─── Example switcher ────────────────────────────────────────────────────────

function switchExample(key) {
  state.example        = key;
  state.offsetX        = 0;
  state.offsetY        = 0;
  state.zoom           = 1.0;
  state.selectedNodeId = null;

  const p = DEFAULT_PARAMS[key];
  radiusInput.value    = p.nodeRadius;
  levelGapInput.value  = p.levelGap;
  sibGapInput.value    = p.siblingGap;
  radiusVal.textContent    = p.nodeRadius;
  levelGapVal.textContent  = p.levelGap;
  sibGapVal.textContent    = p.siblingGap;

  render();
  renderDetails();
}

// ─── Pan (pointer drag) ──────────────────────────────────────────────────────

let pointerDown = false;
let lastPt  = { x: 0, y: 0 };
let startPt = { x: 0, y: 0 };
let didDrag = false;

canvas.addEventListener("pointerdown", e => {
  pointerDown = true;
  didDrag     = false;
  lastPt      = toCanvasPt(e);
  startPt     = { ...lastPt };
  canvas.setPointerCapture(e.pointerId);
  canvas.classList.add("is-dragging");
  e.preventDefault();
});

canvas.addEventListener("pointermove", e => {
  if (!pointerDown) return;
  const pt = toCanvasPt(e);
  if (Math.hypot(pt.x - startPt.x, pt.y - startPt.y) > 4) didDrag = true;
  state.offsetX += Math.round(pt.x - lastPt.x);
  state.offsetY += Math.round(pt.y - lastPt.y);
  lastPt = pt;
  render();
  e.preventDefault();
});

function endPan() {
  pointerDown = false;
  canvas.classList.remove("is-dragging");
}
canvas.addEventListener("pointerup",     endPan);
canvas.addEventListener("pointercancel", endPan);

// ─── Wheel zoom ──────────────────────────────────────────────────────────────

canvas.addEventListener("wheel", e => {
  e.preventDefault();
  const factor   = e.deltaY < 0 ? 1.12 : 1 / 1.12;
  const newZoom  = Math.min(Math.max(state.zoom * factor, 0.15), 8.0);
  const pt       = toCanvasPt(e);
  // Keep the canvas point under the cursor fixed:
  //   pt = worldPos * zoom + offset  →  worldPos = (pt - offset) / zoom
  //   newOffset = pt - worldPos * newZoom
  const ratio    = newZoom / state.zoom;
  state.offsetX  = Math.round(pt.x - (pt.x - state.offsetX) * ratio);
  state.offsetY  = Math.round(pt.y - (pt.y - state.offsetY) * ratio);
  state.zoom     = newZoom;
  render();
}, { passive: false });

// ─── Click to select ─────────────────────────────────────────────────────────

canvas.addEventListener("click", e => {
  if (didDrag) return;
  const pt = toCanvasPt(e);
  try {
    const hit = wasm.pick_tree_node(buildSpec(), pt.x, pt.y);
    state.selectedNodeId = hit?.node_id ?? null;
    render();
    renderDetails();
  } catch (err) {
    console.error("[tree] pick error:", err);
  }
});

// ─── Controls ────────────────────────────────────────────────────────────────

exampleSelect.addEventListener("change", () => switchExample(exampleSelect.value));

function syncSlider(input, display) {
  display.textContent = input.value;
  render();
}
radiusInput.addEventListener("input",   () => syncSlider(radiusInput,   radiusVal));
levelGapInput.addEventListener("input", () => syncSlider(levelGapInput, levelGapVal));
sibGapInput.addEventListener("input",   () => syncSlider(sibGapInput,   sibGapVal));

shapeBtns.forEach(btn => {
  btn.addEventListener("click", () => {
    shapeBtns.forEach(b => b.classList.remove("active"));
    btn.classList.add("active");
    const s = btn.dataset.shape;
    state.shapeOverride = (s === "auto") ? null : s;
    render();
  });
});

labelInsideCheck.addEventListener("change", () => {
  state.labelInside = labelInsideCheck.checked;
  render();
});

prInput.addEventListener("input", () => {
  const oldPR        = state.pixelRatio;
  const newPR        = parseFloat(prInput.value);
  prVal.textContent  = newPR + "×";
  // Scale offsets so the view position is preserved
  const ratio        = newPR / oldPR;
  state.offsetX      = Math.round(state.offsetX * ratio);
  state.offsetY      = Math.round(state.offsetY * ratio);
  state.pixelRatio   = newPR;
  setupCanvas();
  render();
});

resetBtn.addEventListener("click", () => {
  state.offsetX        = 0;
  state.offsetY        = 0;
  state.zoom           = 1.0;
  state.selectedNodeId = null;
  render();
  renderDetails();
});

// ─── Boot ────────────────────────────────────────────────────────────────────

await init();

// Sync pixel-ratio slider to detected device ratio, then set up canvas
prInput.value         = state.pixelRatio;
prVal.textContent     = state.pixelRatio + "×";
setupCanvas();

// Apply default params for the initial example
const p0 = DEFAULT_PARAMS[state.example];
radiusInput.value   = p0.nodeRadius;
levelGapInput.value = p0.levelGap;
sibGapInput.value   = p0.siblingGap;
radiusVal.textContent   = p0.nodeRadius;
levelGapVal.textContent = p0.levelGap;
sibGapVal.textContent   = p0.siblingGap;

render();
renderDetails();

statusPanel.className = "status-panel status-success";
const title = statusPanel.querySelector(".status-title");
const body  = statusPanel.querySelector(".status-body");
title.textContent = "WASM ready.";
body.textContent  = "Drag to pan · click a node to select.";
