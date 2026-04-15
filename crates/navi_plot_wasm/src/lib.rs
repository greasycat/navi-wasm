#[cfg(target_arch = "wasm32")]
mod wasm_impl {
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

    #[derive(Serialize)]
    struct ScatterHit {
        index: usize,
    }

    #[derive(Serialize)]
    struct TreeHit {
        node_id: String,
    }

    #[derive(Serialize)]
    struct LineHit {
        series_index: usize,
        point_index: usize,
    }

    #[derive(Serialize)]
    struct BarHit {
        series_index: usize,
        category_index: usize,
    }

    #[derive(Serialize)]
    struct HeatmapHit {
        row: usize,
        col: usize,
    }

    #[derive(Serialize)]
    struct NetworkHit {
        node_id: String,
    }

    #[derive(Serialize)]
    struct NetworkBadgeHit {
        kind: String,
        node_id: String,
    }

    struct ScatterCanvasSession {
        canvas_id: String,
        session: ScatterSession,
    }

    struct ScatterSessionStore {
        next_handle: u32,
        sessions: BTreeMap<u32, ScatterCanvasSession>,
    }

    impl Default for ScatterSessionStore {
        fn default() -> Self {
            Self {
                next_handle: 1,
                sessions: BTreeMap::new(),
            }
        }
    }

    impl ScatterSessionStore {
        fn insert(&mut self, canvas_id: String, session: ScatterSession) -> u32 {
            loop {
                let handle = if self.next_handle == 0 {
                    1
                } else {
                    self.next_handle
                };
                self.next_handle = handle.wrapping_add(1);

                if self.sessions.contains_key(&handle) {
                    continue;
                }

                self.sessions
                    .insert(handle, ScatterCanvasSession { canvas_id, session });
                return handle;
            }
        }
    }

    struct TreeCanvasSession {
        canvas_id: String,
        session: TreeSession,
    }

    struct TreeSessionStore {
        next_handle: u32,
        sessions: BTreeMap<u32, TreeCanvasSession>,
    }

    impl Default for TreeSessionStore {
        fn default() -> Self {
            Self {
                next_handle: 1,
                sessions: BTreeMap::new(),
            }
        }
    }

    impl TreeSessionStore {
        fn insert(&mut self, canvas_id: String, session: TreeSession) -> u32 {
            loop {
                let handle = if self.next_handle == 0 {
                    1
                } else {
                    self.next_handle
                };
                self.next_handle = handle.wrapping_add(1);

                if self.sessions.contains_key(&handle) {
                    continue;
                }

                self.sessions
                    .insert(handle, TreeCanvasSession { canvas_id, session });
                return handle;
            }
        }
    }

    struct BarCanvasSession {
        canvas_id: String,
        session: BarSession,
    }

    struct BarSessionStore {
        next_handle: u32,
        sessions: BTreeMap<u32, BarCanvasSession>,
    }

    impl Default for BarSessionStore {
        fn default() -> Self {
            Self {
                next_handle: 1,
                sessions: BTreeMap::new(),
            }
        }
    }

    impl BarSessionStore {
        fn insert(&mut self, canvas_id: String, session: BarSession) -> u32 {
            loop {
                let handle = if self.next_handle == 0 {
                    1
                } else {
                    self.next_handle
                };
                self.next_handle = handle.wrapping_add(1);

                if self.sessions.contains_key(&handle) {
                    continue;
                }

                self.sessions
                    .insert(handle, BarCanvasSession { canvas_id, session });
                return handle;
            }
        }
    }

    struct HeatmapCanvasSession {
        canvas_id: String,
        session: HeatmapSession,
    }

    struct HeatmapSessionStore {
        next_handle: u32,
        sessions: BTreeMap<u32, HeatmapCanvasSession>,
    }

    impl Default for HeatmapSessionStore {
        fn default() -> Self {
            Self {
                next_handle: 1,
                sessions: BTreeMap::new(),
            }
        }
    }

    impl HeatmapSessionStore {
        fn insert(&mut self, canvas_id: String, session: HeatmapSession) -> u32 {
            loop {
                let handle = if self.next_handle == 0 {
                    1
                } else {
                    self.next_handle
                };
                self.next_handle = handle.wrapping_add(1);

                if self.sessions.contains_key(&handle) {
                    continue;
                }

                self.sessions
                    .insert(handle, HeatmapCanvasSession { canvas_id, session });
                return handle;
            }
        }
    }

    thread_local! {
        static SCATTER_SESSIONS: RefCell<ScatterSessionStore> =
            RefCell::new(ScatterSessionStore::default());
        static TREE_SESSIONS: RefCell<TreeSessionStore> =
            RefCell::new(TreeSessionStore::default());
        static BAR_SESSIONS: RefCell<BarSessionStore> =
            RefCell::new(BarSessionStore::default());
        static HEATMAP_SESSIONS: RefCell<HeatmapSessionStore> =
            RefCell::new(HeatmapSessionStore::default());
        static GRAPH_IMAGES: RefCell<BTreeMap<String, HtmlImageElement>> =
            RefCell::new(BTreeMap::new());
    }

    /// Convert a generic display message to a plain JS string error.
    /// Used for infrastructure errors (canvas not found, bad session handle, etc.)
    fn js_error(message: impl core::fmt::Display) -> JsValue {
        JsValue::from_str(&message.to_string())
    }

    /// Convert a `PlotError` to a real JS `Error` object with a `.code` property,
    /// so callers can distinguish error types programmatically:
    /// ```js
    /// catch (e) { if (e.code === "EMPTY_SCATTER_DATA") { ... } }
    /// ```
    fn plot_error_to_js(err: PlotError) -> JsValue {
        let e = js_sys::Error::new(&err.to_string());
        let _ = js_sys::Reflect::set(&e, &"code".into(), &JsValue::from_str(err.code()));
        e.into()
    }

    fn unknown_scatter_session(handle: u32) -> JsValue {
        JsValue::from_str(&format!("unknown scatter session handle {handle}"))
    }

    fn unknown_tree_session(handle: u32) -> JsValue {
        JsValue::from_str(&format!("unknown tree session handle {handle}"))
    }

    fn unknown_bar_session(handle: u32) -> JsValue {
        JsValue::from_str(&format!("unknown bar session handle {handle}"))
    }

    fn unknown_heatmap_session(handle: u32) -> JsValue {
        JsValue::from_str(&format!("unknown heatmap session handle {handle}"))
    }

    fn to_js_value<T>(value: &T) -> Result<JsValue, JsValue>
    where
        T: Serialize,
    {
        value
            .serialize(&Serializer::json_compatible())
            .map_err(js_error)
    }

    fn to_network_badge_hit(hit: NetworkPickHit) -> NetworkBadgeHit {
        let kind = match hit.kind {
            NetworkPickKind::Node => "node",
            NetworkPickKind::Toggle => "toggle",
        };
        NetworkBadgeHit {
            kind: kind.to_string(),
            node_id: hit.node_id,
        }
    }

    fn canvas_by_id(canvas_id: &str) -> Result<HtmlCanvasElement, JsValue> {
        let window = window().ok_or_else(|| JsValue::from_str("window is not available"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document is not available"))?;
        let element = document.get_element_by_id(canvas_id).ok_or_else(|| {
            JsValue::from_str(&format!("canvas element '{canvas_id}' was not found"))
        })?;

        element
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str(&format!("element '{canvas_id}' is not an HTML canvas")))
    }

    fn drawing_area(
        canvas_id: &str,
        width: u32,
        height: u32,
    ) -> Result<plotters::drawing::DrawingArea<CanvasBackend, Shift>, JsValue> {
        let canvas = canvas_by_id(canvas_id)?;
        if canvas.width() != width {
            canvas.set_width(width);
        }
        if canvas.height() != height {
            canvas.set_height(height);
        }
        let backend = CanvasBackend::with_canvas_object(canvas).ok_or_else(|| {
            JsValue::from_str("failed to create CanvasBackend from canvas element")
        })?;
        Ok(backend.into_drawing_area())
    }

    fn canvas_2d_context(canvas_id: &str) -> Result<CanvasRenderingContext2d, JsValue> {
        let canvas = canvas_by_id(canvas_id)?;
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| js_error("2d canvas context is not available"))?;
        context
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| js_error("failed to cast 2d canvas context"))
    }

    pub async fn register_graph_image(key: String, src: String) -> Result<(), JsValue> {
        let key = key.trim().to_owned();
        if key.is_empty() {
            return Err(js_error("graph image key must not be empty"));
        }

        let src = src.trim().to_owned();
        if src.is_empty() {
            return Err(js_error("graph image src must not be empty"));
        }

        let image = HtmlImageElement::new()?;
        if !src.starts_with("data:") {
            image.set_cross_origin(Some("anonymous"));
        }
        image.set_src(&src);
        JsFuture::from(image.decode()).await?;

        GRAPH_IMAGES.with(|images| {
            images.borrow_mut().insert(key, image);
        });

        Ok(())
    }

    pub fn has_graph_image(key: &str) -> bool {
        GRAPH_IMAGES.with(|images| images.borrow().contains_key(key))
    }

    pub fn unregister_graph_image(key: &str) -> bool {
        GRAPH_IMAGES.with(|images| images.borrow_mut().remove(key).is_some())
    }

    pub fn clear_graph_images() {
        GRAPH_IMAGES.with(|images| images.borrow_mut().clear());
    }

    fn clip_node_shape(
        context: &CanvasRenderingContext2d,
        shape: &NodeShape,
        center_x: f64,
        center_y: f64,
        radius: f64,
    ) -> Result<(), JsValue> {
        context.begin_path();
        match shape {
            NodeShape::Circle => {
                context.arc(center_x, center_y, radius, 0.0, std::f64::consts::TAU)?;
            }
            NodeShape::Square => {
                context.rect(
                    center_x - radius,
                    center_y - radius,
                    radius * 2.0,
                    radius * 2.0,
                );
            }
            NodeShape::Diamond => {
                context.move_to(center_x, center_y - radius);
                context.line_to(center_x + radius, center_y);
                context.line_to(center_x, center_y + radius);
                context.line_to(center_x - radius, center_y);
                context.close_path();
            }
            NodeShape::Triangle => {
                context.move_to(center_x, center_y - radius);
                context.line_to(center_x + radius, center_y + radius);
                context.line_to(center_x - radius, center_y + radius);
                context.close_path();
            }
        }
        context.clip();
        Ok(())
    }

    fn draw_graph_image(
        context: &CanvasRenderingContext2d,
        image: &HtmlImageElement,
        node: &GraphNodeRenderInfo,
        fit: &NodeMediaFit,
        scale: f64,
    ) -> Result<(), JsValue> {
        let image_width = image.natural_width();
        let image_height = image.natural_height();
        if image_width == 0 || image_height == 0 {
            return Ok(());
        }

        let content_box = f64::from(node.radius.max(1)) * scale * 2.0;
        let box_width = content_box.max(2.0);
        let box_height = content_box.max(2.0);
        let aspect = f64::from(image_width) / f64::from(image_height);

        let (draw_width, draw_height) = match fit {
            NodeMediaFit::Contain => {
                if aspect >= 1.0 {
                    (box_width, box_width / aspect)
                } else {
                    (box_height * aspect, box_height)
                }
            }
            NodeMediaFit::Cover => {
                if aspect >= 1.0 {
                    (box_height * aspect, box_height)
                } else {
                    (box_width, box_width / aspect)
                }
            }
        };

        let x = f64::from(node.center_x) - draw_width / 2.0;
        let y = f64::from(node.center_y) - draw_height / 2.0;

        context.save();
        clip_node_shape(
            context,
            &node.shape,
            f64::from(node.center_x),
            f64::from(node.center_y),
            f64::from(node.radius.max(1)),
        )?;
        context.draw_image_with_html_image_element_and_dw_and_dh(
            image,
            x,
            y,
            draw_width,
            draw_height,
        )?;
        context.restore();
        Ok(())
    }

    fn overlay_graph_images(canvas_id: &str, nodes: &[GraphNodeRenderInfo]) -> Result<(), JsValue> {
        let context = canvas_2d_context(canvas_id)?;

        for node in nodes {
            let Some(media) = node.media.as_ref() else {
                continue;
            };
            let ResolvedNodeMediaKind::Image { image_key, fit, .. } = &media.kind else {
                continue;
            };
            let image = GRAPH_IMAGES.with(|images| images.borrow().get(image_key).cloned());
            let Some(image) = image else {
                continue;
            };
            context.save();
            context.set_global_alpha(node.opacity.clamp(0.0, 1.0));
            draw_graph_image(&context, &image, node, fit, media.scale)?;
            context.restore();
        }

        Ok(())
    }

    fn with_scatter_session_mut<T>(
        handle: u32,
        f: impl FnOnce(&mut ScatterCanvasSession) -> Result<T, JsValue>,
    ) -> Result<T, JsValue> {
        SCATTER_SESSIONS.with(|store| {
            let mut store = store.borrow_mut();
            let session = store
                .sessions
                .get_mut(&handle)
                .ok_or_else(|| unknown_scatter_session(handle))?;
            f(session)
        })
    }

    fn with_tree_session_mut<T>(
        handle: u32,
        f: impl FnOnce(&mut TreeCanvasSession) -> Result<T, JsValue>,
    ) -> Result<T, JsValue> {
        TREE_SESSIONS.with(|store| {
            let mut store = store.borrow_mut();
            let session = store
                .sessions
                .get_mut(&handle)
                .ok_or_else(|| unknown_tree_session(handle))?;
            f(session)
        })
    }

    fn with_bar_session_mut<T>(
        handle: u32,
        f: impl FnOnce(&mut BarCanvasSession) -> Result<T, JsValue>,
    ) -> Result<T, JsValue> {
        BAR_SESSIONS.with(|store| {
            let mut store = store.borrow_mut();
            let session = store
                .sessions
                .get_mut(&handle)
                .ok_or_else(|| unknown_bar_session(handle))?;
            f(session)
        })
    }

    fn with_heatmap_session_mut<T>(
        handle: u32,
        f: impl FnOnce(&mut HeatmapCanvasSession) -> Result<T, JsValue>,
    ) -> Result<T, JsValue> {
        HEATMAP_SESSIONS.with(|store| {
            let mut store = store.borrow_mut();
            let session = store
                .sessions
                .get_mut(&handle)
                .ok_or_else(|| unknown_heatmap_session(handle))?;
            f(session)
        })
    }

    pub fn create_scatter_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
        let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
        let session = ScatterSession::new(spec).map_err(plot_error_to_js)?;
        let _ = canvas_by_id(canvas_id)?;

        SCATTER_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
    }

    pub fn render_scatter_session(handle: u32) -> Result<(), JsValue> {
        with_scatter_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry.session.render_on(root).map_err(plot_error_to_js)
        })
    }

    pub fn pan_scatter_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
        with_scatter_session_mut(handle, |entry| {
            entry.session.pan(delta_x, delta_y);
            Ok(())
        })
    }

    pub fn zoom_scatter_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), JsValue> {
        with_scatter_session_mut(handle, |entry| {
            entry
                .session
                .zoom_at(canvas_x, canvas_y, factor)
                .map_err(plot_error_to_js)
        })
    }

    pub fn pick_scatter_point_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        with_scatter_session_mut(handle, |entry| {
            let hit = entry.session.pick_point(canvas_x, canvas_y);
            to_js_value(&hit.map(|index| ScatterHit { index }))
        })
    }

    pub fn set_scatter_selection(handle: u32, index: Option<u32>) -> Result<(), JsValue> {
        with_scatter_session_mut(handle, |entry| {
            entry
                .session
                .set_selection(index.map(|value| value as usize));
            Ok(())
        })
    }

    pub fn destroy_scatter_session(handle: u32) -> Result<(), JsValue> {
        SCATTER_SESSIONS.with(|store| {
            let removed = store.borrow_mut().sessions.remove(&handle);
            if removed.is_some() {
                Ok(())
            } else {
                Err(unknown_scatter_session(handle))
            }
        })
    }

    pub fn render_scatter(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
        let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
        let root = drawing_area(canvas_id, spec.width, spec.height)?;
        render_scatter_on(root, &spec).map_err(plot_error_to_js)
    }

    pub fn render_tree(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
        let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
        let root = drawing_area(canvas_id, spec.width, spec.height)?;
        render_tree_on(root, &spec).map_err(plot_error_to_js)?;
        let nodes = tree_render_nodes(&spec).map_err(plot_error_to_js)?;
        overlay_graph_images(canvas_id, &nodes)
    }

    pub fn create_tree_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
        let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
        let session = TreeSession::new(spec).map_err(plot_error_to_js)?;
        let _ = canvas_by_id(canvas_id)?;
        TREE_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
    }

    pub fn render_tree_session(handle: u32) -> Result<(), JsValue> {
        with_tree_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry.session.render_on(root).map_err(plot_error_to_js)?;
            let nodes = entry.session.render_nodes();
            overlay_graph_images(&entry.canvas_id, &nodes)
        })
    }

    pub fn render_tree_session_transition(handle: u32, progress: f64) -> Result<(), JsValue> {
        with_tree_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry
                .session
                .render_transition_on(root, progress)
                .map_err(plot_error_to_js)?;
            let nodes = entry.session.render_transition_nodes(progress);
            overlay_graph_images(&entry.canvas_id, &nodes)
        })
    }

    pub fn pan_tree_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
        with_tree_session_mut(handle, |entry| {
            entry.session.pan(delta_x, delta_y);
            Ok(())
        })
    }

    pub fn zoom_tree_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), JsValue> {
        with_tree_session_mut(handle, |entry| {
            entry
                .session
                .zoom_at(canvas_x, canvas_y, factor)
                .map_err(plot_error_to_js)
        })
    }

    pub fn pick_tree_node_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        with_tree_session_mut(handle, |entry| {
            let hit = entry.session.pick_node(canvas_x, canvas_y);
            to_js_value(&hit.map(|node_id| TreeHit { node_id }))
        })
    }

    pub fn set_tree_selection(handle: u32, node_id: Option<String>) -> Result<(), JsValue> {
        with_tree_session_mut(handle, |entry| {
            entry.session.set_selection(node_id);
            Ok(())
        })
    }

    pub fn toggle_tree_node_collapsed_session(
        handle: u32,
        node_id: String,
    ) -> Result<bool, JsValue> {
        with_tree_session_mut(handle, |entry| Ok(entry.session.toggle_collapse(&node_id)))
    }

    pub fn set_tree_node_collapsed_session(
        handle: u32,
        node_id: String,
        collapsed: bool,
    ) -> Result<(), JsValue> {
        with_tree_session_mut(handle, |entry| {
            entry.session.set_collapsed(&node_id, collapsed);
            Ok(())
        })
    }

    pub fn destroy_tree_session(handle: u32) -> Result<(), JsValue> {
        TREE_SESSIONS.with(|store| {
            store
                .borrow_mut()
                .sessions
                .remove(&handle)
                .map(|_| ())
                .ok_or_else(|| unknown_tree_session(handle))
        })
    }

    pub fn pan_scatter(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
        let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
        let next = pan_scatter_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
        to_js_value(&next)
    }

    pub fn pan_tree(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
        let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
        let next = pan_tree_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
        to_js_value(&next)
    }

    pub fn pick_scatter_point(
        spec: JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
        let hit = core_pick_scatter_point(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
        to_js_value(&hit.map(|index| ScatterHit { index }))
    }

    pub fn pick_tree_node(spec: JsValue, canvas_x: f64, canvas_y: f64) -> Result<JsValue, JsValue> {
        let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
        let hit = core_pick_tree_node(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
        to_js_value(&hit.map(|node_id| TreeHit { node_id }))
    }

    // ── Line chart session store ─────────────────────────────────────────────

    struct LineCanvasSession {
        canvas_id: String,
        session: LineSession,
    }

    struct LineSessionStore {
        next_handle: u32,
        sessions: BTreeMap<u32, LineCanvasSession>,
    }

    impl Default for LineSessionStore {
        fn default() -> Self {
            Self {
                next_handle: 1,
                sessions: BTreeMap::new(),
            }
        }
    }

    impl LineSessionStore {
        fn insert(&mut self, canvas_id: String, session: LineSession) -> u32 {
            loop {
                let handle = if self.next_handle == 0 {
                    1
                } else {
                    self.next_handle
                };
                self.next_handle = handle.wrapping_add(1);
                if !self.sessions.contains_key(&handle) {
                    self.sessions
                        .insert(handle, LineCanvasSession { canvas_id, session });
                    return handle;
                }
            }
        }
    }

    thread_local! {
        static LINE_SESSIONS: RefCell<LineSessionStore> =
            RefCell::new(LineSessionStore::default());
    }

    fn unknown_line_session(handle: u32) -> JsValue {
        JsValue::from_str(&format!("unknown line session handle {handle}"))
    }

    fn with_line_session_mut<T>(
        handle: u32,
        f: impl FnOnce(&mut LineCanvasSession) -> Result<T, JsValue>,
    ) -> Result<T, JsValue> {
        LINE_SESSIONS.with(|store| {
            let mut store = store.borrow_mut();
            let session = store
                .sessions
                .get_mut(&handle)
                .ok_or_else(|| unknown_line_session(handle))?;
            f(session)
        })
    }

    pub fn create_line_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
        let spec: LinePlotSpec = from_value(spec).map_err(js_error)?;
        let session = LineSession::new(spec).map_err(plot_error_to_js)?;
        let _ = canvas_by_id(canvas_id)?;
        LINE_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
    }

    pub fn render_line_session(handle: u32) -> Result<(), JsValue> {
        with_line_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry.session.render_on(root).map_err(plot_error_to_js)
        })
    }

    pub fn pan_line_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
        with_line_session_mut(handle, |entry| {
            entry.session.pan(delta_x, delta_y);
            Ok(())
        })
    }

    pub fn zoom_line_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), JsValue> {
        with_line_session_mut(handle, |entry| {
            entry
                .session
                .zoom_at(canvas_x, canvas_y, factor)
                .map_err(plot_error_to_js)
        })
    }

    pub fn pick_line_point_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        with_line_session_mut(handle, |entry| {
            let hit = entry.session.pick_point(canvas_x, canvas_y);
            to_js_value(&hit.map(|[si, pi]| LineHit {
                series_index: si,
                point_index: pi,
            }))
        })
    }

    pub fn set_line_selection(
        handle: u32,
        series_index: Option<u32>,
        point_index: Option<u32>,
    ) -> Result<(), JsValue> {
        with_line_session_mut(handle, |entry| {
            let sel = match (series_index, point_index) {
                (Some(si), Some(pi)) => Some([si as usize, pi as usize]),
                _ => None,
            };
            entry.session.set_selection(sel);
            Ok(())
        })
    }

    pub fn destroy_line_session(handle: u32) -> Result<(), JsValue> {
        LINE_SESSIONS.with(|store| {
            store
                .borrow_mut()
                .sessions
                .remove(&handle)
                .map(|_| ())
                .ok_or_else(|| unknown_line_session(handle))
        })
    }

    pub fn render_line(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
        let spec: LinePlotSpec = from_value(spec).map_err(js_error)?;
        let root = drawing_area(canvas_id, spec.width, spec.height)?;
        render_line_on(root, &spec).map_err(plot_error_to_js)
    }

    pub fn pan_line(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
        let spec: LinePlotSpec = from_value(spec).map_err(js_error)?;
        let next = pan_line_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
        to_js_value(&next)
    }

    pub fn pick_line_point(
        spec: JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        let spec: LinePlotSpec = from_value(spec).map_err(js_error)?;
        let hit = core_pick_line_point(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
        to_js_value(&hit.map(|[si, pi]| LineHit {
            series_index: si,
            point_index: pi,
        }))
    }

    // ── Bar chart session store ──────────────────────────────────────────────

    pub fn create_bar_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
        let spec: BarChartSpec = from_value(spec).map_err(js_error)?;
        let session = BarSession::new(spec).map_err(plot_error_to_js)?;
        let _ = canvas_by_id(canvas_id)?;
        BAR_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
    }

    pub fn render_bar_session(handle: u32) -> Result<(), JsValue> {
        with_bar_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry.session.render_on(root).map_err(plot_error_to_js)
        })
    }

    pub fn zoom_bar_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), JsValue> {
        with_bar_session_mut(handle, |entry| {
            entry
                .session
                .zoom_at(canvas_x, canvas_y, factor)
                .map_err(plot_error_to_js)
        })
    }

    pub fn pick_bar_session(handle: u32, canvas_x: f64, canvas_y: f64) -> Result<JsValue, JsValue> {
        with_bar_session_mut(handle, |entry| {
            let hit = entry.session.pick_bar(canvas_x, canvas_y);
            to_js_value(&hit.map(|[series_index, category_index]| BarHit {
                series_index,
                category_index,
            }))
        })
    }

    pub fn set_bar_selection(
        handle: u32,
        series_index: Option<u32>,
        category_index: Option<u32>,
    ) -> Result<(), JsValue> {
        with_bar_session_mut(handle, |entry| {
            let selection = match (series_index, category_index) {
                (Some(series_index), Some(category_index)) => {
                    Some([series_index as usize, category_index as usize])
                }
                _ => None,
            };
            entry.session.set_selection(selection);
            Ok(())
        })
    }

    pub fn destroy_bar_session(handle: u32) -> Result<(), JsValue> {
        BAR_SESSIONS.with(|store| {
            store
                .borrow_mut()
                .sessions
                .remove(&handle)
                .map(|_| ())
                .ok_or_else(|| unknown_bar_session(handle))
        })
    }

    // ── Bar chart (one-shot) ─────────────────────────────────────────────────

    pub fn render_bar(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
        let spec: BarChartSpec = from_value(spec).map_err(js_error)?;
        let root = drawing_area(canvas_id, spec.width, spec.height)?;
        render_bar_on(root, &spec).map_err(plot_error_to_js)
    }

    pub fn pick_bar(spec: JsValue, canvas_x: f64, canvas_y: f64) -> Result<JsValue, JsValue> {
        let spec: BarChartSpec = from_value(spec).map_err(js_error)?;
        let hit = core_pick_bar(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
        to_js_value(&hit.map(|[si, ci]| BarHit {
            series_index: si,
            category_index: ci,
        }))
    }

    // ── Heatmap session store ────────────────────────────────────────────────

    pub fn create_heatmap_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
        let spec: HeatmapSpec = from_value(spec).map_err(js_error)?;
        let session = HeatmapSession::new(spec).map_err(plot_error_to_js)?;
        let _ = canvas_by_id(canvas_id)?;
        HEATMAP_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
    }

    pub fn render_heatmap_session(handle: u32) -> Result<(), JsValue> {
        with_heatmap_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry.session.render_on(root).map_err(plot_error_to_js)
        })
    }

    pub fn zoom_heatmap_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), JsValue> {
        with_heatmap_session_mut(handle, |entry| {
            entry
                .session
                .zoom_at(canvas_x, canvas_y, factor)
                .map_err(plot_error_to_js)
        })
    }

    pub fn pick_heatmap_cell_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        with_heatmap_session_mut(handle, |entry| {
            let hit = entry.session.pick_cell(canvas_x, canvas_y);
            to_js_value(&hit.map(|[row, col]| HeatmapHit { row, col }))
        })
    }

    pub fn set_heatmap_selection(
        handle: u32,
        row: Option<u32>,
        col: Option<u32>,
    ) -> Result<(), JsValue> {
        with_heatmap_session_mut(handle, |entry| {
            let selection = match (row, col) {
                (Some(row), Some(col)) => Some([row as usize, col as usize]),
                _ => None,
            };
            entry.session.set_selection(selection);
            Ok(())
        })
    }

    pub fn destroy_heatmap_session(handle: u32) -> Result<(), JsValue> {
        HEATMAP_SESSIONS.with(|store| {
            store
                .borrow_mut()
                .sessions
                .remove(&handle)
                .map(|_| ())
                .ok_or_else(|| unknown_heatmap_session(handle))
        })
    }

    // ── Heatmap (one-shot) ───────────────────────────────────────────────────

    pub fn render_heatmap(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
        let spec: HeatmapSpec = from_value(spec).map_err(js_error)?;
        let root = drawing_area(canvas_id, spec.width, spec.height)?;
        render_heatmap_on(root, &spec).map_err(plot_error_to_js)
    }

    pub fn pick_heatmap_cell(
        spec: JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        let spec: HeatmapSpec = from_value(spec).map_err(js_error)?;
        let hit = core_pick_heatmap_cell(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
        to_js_value(&hit.map(|[row, col]| HeatmapHit { row, col }))
    }

    // ── Network session store ────────────────────────────────────────────────

    struct NetworkCanvasSession {
        canvas_id: String,
        session: NetworkSession,
    }

    struct NetworkSessionStore {
        next_handle: u32,
        sessions: BTreeMap<u32, NetworkCanvasSession>,
    }

    impl Default for NetworkSessionStore {
        fn default() -> Self {
            Self {
                next_handle: 1,
                sessions: BTreeMap::new(),
            }
        }
    }

    impl NetworkSessionStore {
        fn insert(&mut self, canvas_id: String, session: NetworkSession) -> u32 {
            loop {
                let handle = if self.next_handle == 0 {
                    1
                } else {
                    self.next_handle
                };
                self.next_handle = handle.wrapping_add(1);
                if !self.sessions.contains_key(&handle) {
                    self.sessions
                        .insert(handle, NetworkCanvasSession { canvas_id, session });
                    return handle;
                }
            }
        }
    }

    thread_local! {
        static NETWORK_SESSIONS: RefCell<NetworkSessionStore> =
            RefCell::new(NetworkSessionStore::default());
    }

    fn unknown_network_session(handle: u32) -> JsValue {
        JsValue::from_str(&format!("unknown network session handle {handle}"))
    }

    fn with_network_session_mut<T>(
        handle: u32,
        f: impl FnOnce(&mut NetworkCanvasSession) -> Result<T, JsValue>,
    ) -> Result<T, JsValue> {
        NETWORK_SESSIONS.with(|store| {
            let mut store = store.borrow_mut();
            let session = store
                .sessions
                .get_mut(&handle)
                .ok_or_else(|| unknown_network_session(handle))?;
            f(session)
        })
    }

    pub fn create_network_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
        let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
        let session = NetworkSession::new(spec).map_err(plot_error_to_js)?;
        let _ = canvas_by_id(canvas_id)?;
        NETWORK_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
    }

    pub fn update_network_session(handle: u32, spec: JsValue) -> Result<(), JsValue> {
        let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
        with_network_session_mut(handle, |entry| {
            entry.session.update_spec(spec).map_err(plot_error_to_js)
        })
    }

    pub fn render_network_session(handle: u32) -> Result<(), JsValue> {
        with_network_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry.session.render_on(root).map_err(plot_error_to_js)?;
            let nodes = entry.session.render_nodes();
            overlay_graph_images(&entry.canvas_id, &nodes)
        })
    }

    pub fn render_network_transition_session(handle: u32, progress: f64) -> Result<(), JsValue> {
        with_network_session_mut(handle, |entry| {
            let root = drawing_area(
                &entry.canvas_id,
                entry.session.width(),
                entry.session.height(),
            )?;
            entry
                .session
                .render_transition_on(root, progress)
                .map_err(plot_error_to_js)?;
            let nodes = entry.session.render_transition_nodes(progress);
            overlay_graph_images(&entry.canvas_id, &nodes)
        })
    }

    pub fn pan_network_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
        with_network_session_mut(handle, |entry| {
            entry.session.pan(delta_x, delta_y);
            Ok(())
        })
    }

    pub fn zoom_network_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), JsValue> {
        with_network_session_mut(handle, |entry| {
            entry
                .session
                .zoom_at(canvas_x, canvas_y, factor)
                .map_err(plot_error_to_js)
        })
    }

    pub fn pick_network_node_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        with_network_session_mut(handle, |entry| {
            let hit = entry.session.pick_node(canvas_x, canvas_y);
            to_js_value(&hit.map(|node_id| NetworkHit { node_id }))
        })
    }

    pub fn pick_network_hit_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        with_network_session_mut(handle, |entry| {
            let hit = entry
                .session
                .pick(canvas_x, canvas_y)
                .map(to_network_badge_hit);
            to_js_value(&hit)
        })
    }

    pub fn set_network_selection(handle: u32, node_id: Option<String>) -> Result<(), JsValue> {
        with_network_session_mut(handle, |entry| {
            entry.session.set_selection(node_id.clone());
            Ok(())
        })
    }

    pub fn get_network_view_session(handle: u32) -> Result<JsValue, JsValue> {
        with_network_session_mut(handle, |entry| to_js_value(&entry.session.view()))
    }

    pub fn set_network_view_session(handle: u32, view: JsValue) -> Result<(), JsValue> {
        let view: NetworkView = from_value(view).map_err(js_error)?;
        with_network_session_mut(handle, |entry| {
            entry.session.set_view(view).map_err(plot_error_to_js)
        })
    }

    pub fn compute_network_focus_view_session(
        handle: u32,
        node_id: String,
        options: Option<JsValue>,
    ) -> Result<JsValue, JsValue> {
        let options = match options {
            Some(options) => Some(from_value::<NetworkFocusOptions>(options).map_err(js_error)?),
            None => None,
        };
        with_network_session_mut(handle, |entry| {
            to_js_value(&entry.session.compute_focus_view(&node_id, options))
        })
    }

    pub fn has_network_transition_session(handle: u32) -> Result<bool, JsValue> {
        with_network_session_mut(handle, |entry| Ok(entry.session.has_transition()))
    }

    pub fn clear_network_transition_session(handle: u32) -> Result<(), JsValue> {
        with_network_session_mut(handle, |entry| {
            entry.session.clear_transition();
            Ok(())
        })
    }

    pub fn destroy_network_session(handle: u32) -> Result<(), JsValue> {
        NETWORK_SESSIONS.with(|store| {
            store
                .borrow_mut()
                .sessions
                .remove(&handle)
                .map(|_| ())
                .ok_or_else(|| unknown_network_session(handle))
        })
    }

    pub fn render_network(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
        let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
        let root = drawing_area(canvas_id, spec.width, spec.height)?;
        render_network_on(root, &spec).map_err(plot_error_to_js)?;
        let nodes = network_render_nodes(&spec).map_err(plot_error_to_js)?;
        overlay_graph_images(canvas_id, &nodes)
    }

    pub fn pan_network(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
        let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
        let next = pan_network_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
        to_js_value(&next)
    }

    pub fn pick_network_node(
        spec: JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
        let hit = core_pick_network_node(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
        to_js_value(&hit.map(|node_id| NetworkHit { node_id }))
    }

    pub fn pick_network_hit(
        spec: JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<JsValue, JsValue> {
        let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
        let hit = core_pick_network_hit(&spec, canvas_x, canvas_y)
            .map_err(plot_error_to_js)?
            .map(to_network_badge_hit);
        to_js_value(&hit)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn create_scatter_session(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    wasm_impl::create_scatter_session(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_scatter_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_scatter_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_scatter_session(
    handle: u32,
    delta_x: f64,
    delta_y: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::pan_scatter_session(handle, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn zoom_scatter_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
    factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::zoom_scatter_session(handle, canvas_x, canvas_y, factor)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_scatter_point_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_scatter_point_session(handle, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_scatter_selection(handle: u32, index: Option<u32>) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_scatter_selection(handle, index)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn destroy_scatter_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::destroy_scatter_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_scatter(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_scatter(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_tree(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_tree(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn create_tree_session(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    wasm_impl::create_tree_session(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_tree_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_tree_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_tree_session_transition(
    handle: u32,
    progress: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_tree_session_transition(handle, progress)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_tree_session(
    handle: u32,
    delta_x: f64,
    delta_y: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::pan_tree_session(handle, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn zoom_tree_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
    factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::zoom_tree_session(handle, canvas_x, canvas_y, factor)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_tree_node_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_tree_node_session(handle, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_tree_selection(
    handle: u32,
    node_id: Option<String>,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_tree_selection(handle, node_id)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn toggle_tree_node_collapsed_session(
    handle: u32,
    node_id: String,
) -> Result<bool, wasm_bindgen::JsValue> {
    wasm_impl::toggle_tree_node_collapsed_session(handle, node_id)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_tree_node_collapsed_session(
    handle: u32,
    node_id: String,
    collapsed: bool,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_tree_node_collapsed_session(handle, node_id, collapsed)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn destroy_tree_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::destroy_tree_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn register_graph_image(key: String, src: String) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::register_graph_image(key, src).await
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn has_graph_image(key: &str) -> bool {
    wasm_impl::has_graph_image(key)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn unregister_graph_image(key: &str) -> bool {
    wasm_impl::unregister_graph_image(key)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn clear_graph_images() {
    wasm_impl::clear_graph_images()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_scatter(
    spec: wasm_bindgen::JsValue,
    delta_x: f64,
    delta_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pan_scatter(spec, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_tree(
    spec: wasm_bindgen::JsValue,
    delta_x: f64,
    delta_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pan_tree(spec, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_scatter_point(
    spec: wasm_bindgen::JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_scatter_point(spec, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_tree_node(
    spec: wasm_bindgen::JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_tree_node(spec, canvas_x, canvas_y)
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> wasm_bindgen::JsValue {
    wasm_bindgen::JsValue::from_str(
        "navi_plot_wasm is only available for the wasm32-unknown-unknown target",
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_scatter_session(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_scatter_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pan_scatter_session(
    _handle: u32,
    _delta_x: f64,
    _delta_y: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_scatter_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
    _factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_scatter_point_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_scatter_selection(
    _handle: u32,
    _index: Option<u32>,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn destroy_scatter_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_scatter(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_tree(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_tree_session(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_tree_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_tree_session_transition(
    _handle: u32,
    _progress: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pan_tree_session(
    _handle: u32,
    _delta_x: f64,
    _delta_y: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_tree_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
    _factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_tree_node_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_tree_selection(
    _handle: u32,
    _node_id: Option<String>,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn toggle_tree_node_collapsed_session(
    _handle: u32,
    _node_id: String,
) -> Result<bool, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_tree_node_collapsed_session(
    _handle: u32,
    _node_id: String,
    _collapsed: bool,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn destroy_tree_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn register_graph_image(_key: String, _src: String) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn has_graph_image(_key: &str) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn unregister_graph_image(_key: &str) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_graph_images() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn pan_scatter(
    _spec: wasm_bindgen::JsValue,
    _delta_x: f64,
    _delta_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pan_tree(
    _spec: wasm_bindgen::JsValue,
    _delta_x: f64,
    _delta_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_scatter_point(
    _spec: wasm_bindgen::JsValue,
    _canvas_x: f64,
    _canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_tree_node(
    _spec: wasm_bindgen::JsValue,
    _canvas_x: f64,
    _canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

// ── Line chart ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn create_line_session(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    wasm_impl::create_line_session(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_line_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_line_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_line_session(
    handle: u32,
    delta_x: f64,
    delta_y: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::pan_line_session(handle, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn zoom_line_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
    factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::zoom_line_session(handle, canvas_x, canvas_y, factor)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_line_point_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_line_point_session(handle, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_line_selection(
    handle: u32,
    series_index: Option<u32>,
    point_index: Option<u32>,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_line_selection(handle, series_index, point_index)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn destroy_line_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::destroy_line_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_line(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_line(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_line(
    spec: wasm_bindgen::JsValue,
    delta_x: f64,
    delta_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pan_line(spec, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_line_point(
    spec: wasm_bindgen::JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_line_point(spec, canvas_x, canvas_y)
}

// ── Bar chart ────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn create_bar_session(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    wasm_impl::create_bar_session(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_bar_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_bar_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn zoom_bar_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
    factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::zoom_bar_session(handle, canvas_x, canvas_y, factor)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_bar_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_bar_session(handle, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_bar_selection(
    handle: u32,
    series_index: Option<u32>,
    category_index: Option<u32>,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_bar_selection(handle, series_index, category_index)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn destroy_bar_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::destroy_bar_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_bar(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_bar(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_bar(
    spec: wasm_bindgen::JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_bar(spec, canvas_x, canvas_y)
}

// ── Heatmap ──────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn create_heatmap_session(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    wasm_impl::create_heatmap_session(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_heatmap_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_heatmap_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn zoom_heatmap_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
    factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::zoom_heatmap_session(handle, canvas_x, canvas_y, factor)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_heatmap_cell_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_heatmap_cell_session(handle, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_heatmap_selection(
    handle: u32,
    row: Option<u32>,
    col: Option<u32>,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_heatmap_selection(handle, row, col)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn destroy_heatmap_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::destroy_heatmap_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_heatmap(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_heatmap(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_heatmap_cell(
    spec: wasm_bindgen::JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_heatmap_cell(spec, canvas_x, canvas_y)
}

// ── Network / DAG ────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn create_network_session(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    wasm_impl::create_network_session(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn update_network_session(
    handle: u32,
    spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::update_network_session(handle, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_network_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_network_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_network_transition_session(
    handle: u32,
    progress: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_network_transition_session(handle, progress)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_network_session(
    handle: u32,
    delta_x: f64,
    delta_y: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::pan_network_session(handle, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn zoom_network_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
    factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::zoom_network_session(handle, canvas_x, canvas_y, factor)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_network_node_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_network_node_session(handle, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_network_hit_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_network_hit_session(handle, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_network_selection(
    handle: u32,
    node_id: Option<String>,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_network_selection(handle, node_id)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn get_network_view_session(
    handle: u32,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::get_network_view_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_network_view_session(
    handle: u32,
    view: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::set_network_view_session(handle, view)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compute_network_focus_view_session(
    handle: u32,
    node_id: String,
    options: Option<wasm_bindgen::JsValue>,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::compute_network_focus_view_session(handle, node_id, options)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn has_network_transition_session(handle: u32) -> Result<bool, wasm_bindgen::JsValue> {
    wasm_impl::has_network_transition_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn clear_network_transition_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::clear_network_transition_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn destroy_network_session(handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::destroy_network_session(handle)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_network(
    canvas_id: &str,
    spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    wasm_impl::render_network(canvas_id, spec)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pan_network(
    spec: wasm_bindgen::JsValue,
    delta_x: f64,
    delta_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pan_network(spec, delta_x, delta_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_network_node(
    spec: wasm_bindgen::JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_network_node(spec, canvas_x, canvas_y)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn pick_network_hit(
    spec: wasm_bindgen::JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    wasm_impl::pick_network_hit(spec, canvas_x, canvas_y)
}

// ── non-wasm32 stubs ─────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub fn create_line_session(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_line_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pan_line_session(_handle: u32, _dx: f64, _dy: f64) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_line_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
    _factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_line_point_session(
    _handle: u32,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn set_line_selection(
    _handle: u32,
    _si: Option<u32>,
    _pi: Option<u32>,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn destroy_line_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_line(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pan_line(
    _spec: wasm_bindgen::JsValue,
    _dx: f64,
    _dy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_line_point(
    _spec: wasm_bindgen::JsValue,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_bar_session(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_bar_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_bar_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
    _factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_bar_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn set_bar_selection(
    _handle: u32,
    _series_index: Option<u32>,
    _category_index: Option<u32>,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn destroy_bar_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_bar(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_bar(
    _spec: wasm_bindgen::JsValue,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_heatmap_session(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_heatmap_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_heatmap_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
    _factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_heatmap_cell_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn set_heatmap_selection(
    _handle: u32,
    _row: Option<u32>,
    _col: Option<u32>,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn destroy_heatmap_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_heatmap(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_heatmap_cell(
    _spec: wasm_bindgen::JsValue,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_network_session(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<u32, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn update_network_session(
    _handle: u32,
    _spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_network_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_network_transition_session(
    _handle: u32,
    _progress: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pan_network_session(_handle: u32, _dx: f64, _dy: f64) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn zoom_network_session(
    _handle: u32,
    _canvas_x: f64,
    _canvas_y: f64,
    _factor: f64,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_network_node_session(
    _handle: u32,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_network_hit_session(
    _handle: u32,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn set_network_selection(
    _handle: u32,
    _node_id: Option<String>,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn get_network_view_session(
    _handle: u32,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn set_network_view_session(
    _handle: u32,
    _view: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn compute_network_focus_view_session(
    _handle: u32,
    _node_id: String,
    _options: Option<wasm_bindgen::JsValue>,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn has_network_transition_session(_handle: u32) -> Result<bool, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn clear_network_transition_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn destroy_network_session(_handle: u32) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn render_network(
    _canvas_id: &str,
    _spec: wasm_bindgen::JsValue,
) -> Result<(), wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pan_network(
    _spec: wasm_bindgen::JsValue,
    _dx: f64,
    _dy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_network_node(
    _spec: wasm_bindgen::JsValue,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
#[cfg(not(target_arch = "wasm32"))]
pub fn pick_network_hit(
    _spec: wasm_bindgen::JsValue,
    _cx: f64,
    _cy: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    Err(unsupported())
}
