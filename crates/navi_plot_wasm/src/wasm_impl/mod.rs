use js_sys;
use navi_plot_core::{
    network_render_nodes, pan_line_spec, pan_network_spec, pan_scatter_spec, pan_tree_spec,
    pick_bar as core_pick_bar, pick_heatmap_cell as core_pick_heatmap_cell,
    pick_line_point as core_pick_line_point, pick_network_hit as core_pick_network_hit,
    pick_network_node as core_pick_network_node, pick_scatter_point as core_pick_scatter_point,
    pick_tree_node as core_pick_tree_node, render_bar_on, render_heatmap_on, render_line_on,
    render_network_on, render_scatter_on, render_tree_on, tree_render_nodes, BarChartSpec,
    BarSession, GraphNodeRenderInfo, HeatmapSession, HeatmapSpec, LinePlotSpec, LineSession,
    NetworkFocusOptions, NetworkPickHit, NetworkPickKind, NetworkPlotSpec, NetworkSession,
    NetworkView, NodeMediaFit, NodeShape, PlotError, ResolvedNodeMediaKind, ScatterPlotSpec,
    ScatterSession, TreePlotSpec, TreeSession,
};
use plotters::coord::Shift;
use plotters::prelude::IntoDrawingArea;
use plotters_canvas::CanvasBackend;
use serde::Serialize;
use serde_wasm_bindgen::{from_value, Serializer};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

mod charts;
mod network;
mod shared;

pub(crate) use self::charts::*;
pub(crate) use self::network::*;
pub(crate) use self::shared::*;
