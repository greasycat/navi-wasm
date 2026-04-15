use super::*;

#[derive(Serialize)]
pub(crate) struct ScatterHit {
    pub(crate) index: usize,
}

#[derive(Serialize)]
pub(crate) struct TreeHit {
    pub(crate) node_id: String,
}

#[derive(Serialize)]
pub(crate) struct LineHit {
    pub(crate) series_index: usize,
    pub(crate) point_index: usize,
}

#[derive(Serialize)]
pub(crate) struct BarHit {
    pub(crate) series_index: usize,
    pub(crate) category_index: usize,
}

#[derive(Serialize)]
pub(crate) struct HeatmapHit {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Serialize)]
pub(crate) struct NetworkHit {
    pub(crate) node_id: String,
}

#[derive(Serialize)]
pub(crate) struct NetworkBadgeHit {
    pub(crate) kind: String,
    pub(crate) node_id: String,
}

pub(crate) struct ScatterCanvasSession {
    pub(crate) canvas_id: String,
    pub(crate) session: ScatterSession,
}

pub(crate) struct ScatterSessionStore {
    pub(crate) next_handle: u32,
    pub(crate) sessions: BTreeMap<u32, ScatterCanvasSession>,
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
    pub(crate) fn insert(&mut self, canvas_id: String, session: ScatterSession) -> u32 {
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

pub(crate) struct TreeCanvasSession {
    pub(crate) canvas_id: String,
    pub(crate) session: TreeSession,
}

pub(crate) struct TreeSessionStore {
    pub(crate) next_handle: u32,
    pub(crate) sessions: BTreeMap<u32, TreeCanvasSession>,
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
    pub(crate) fn insert(&mut self, canvas_id: String, session: TreeSession) -> u32 {
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

pub(crate) struct BarCanvasSession {
    pub(crate) canvas_id: String,
    pub(crate) session: BarSession,
}

pub(crate) struct BarSessionStore {
    pub(crate) next_handle: u32,
    pub(crate) sessions: BTreeMap<u32, BarCanvasSession>,
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
    pub(crate) fn insert(&mut self, canvas_id: String, session: BarSession) -> u32 {
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

pub(crate) struct HeatmapCanvasSession {
    pub(crate) canvas_id: String,
    pub(crate) session: HeatmapSession,
}

pub(crate) struct HeatmapSessionStore {
    pub(crate) next_handle: u32,
    pub(crate) sessions: BTreeMap<u32, HeatmapCanvasSession>,
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
    pub(crate) fn insert(&mut self, canvas_id: String, session: HeatmapSession) -> u32 {
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
    pub(crate) static SCATTER_SESSIONS: RefCell<ScatterSessionStore> =
        RefCell::new(ScatterSessionStore::default());
    pub(crate) static TREE_SESSIONS: RefCell<TreeSessionStore> =
        RefCell::new(TreeSessionStore::default());
    pub(crate) static BAR_SESSIONS: RefCell<BarSessionStore> =
        RefCell::new(BarSessionStore::default());
    pub(crate) static HEATMAP_SESSIONS: RefCell<HeatmapSessionStore> =
        RefCell::new(HeatmapSessionStore::default());
    pub(crate) static GRAPH_IMAGES: RefCell<BTreeMap<String, HtmlImageElement>> =
        RefCell::new(BTreeMap::new());
}

/// Convert a generic display message to a plain JS string error.
/// Used for infrastructure errors (canvas not found, bad session handle, etc.)
pub(crate) fn js_error(message: impl core::fmt::Display) -> JsValue {
    JsValue::from_str(&message.to_string())
}

/// Convert a `PlotError` to a real JS `Error` object with a `.code` property,
/// so callers can distinguish error types programmatically:
/// ```js
/// catch (e) { if (e.code === "EMPTY_SCATTER_DATA") { ... } }
/// ```
pub(crate) fn plot_error_to_js(err: PlotError) -> JsValue {
    let e = js_sys::Error::new(&err.to_string());
    let _ = js_sys::Reflect::set(&e, &"code".into(), &JsValue::from_str(err.code()));
    e.into()
}

pub(crate) fn unknown_scatter_session(handle: u32) -> JsValue {
    JsValue::from_str(&format!("unknown scatter session handle {handle}"))
}

pub(crate) fn unknown_tree_session(handle: u32) -> JsValue {
    JsValue::from_str(&format!("unknown tree session handle {handle}"))
}

pub(crate) fn unknown_bar_session(handle: u32) -> JsValue {
    JsValue::from_str(&format!("unknown bar session handle {handle}"))
}

pub(crate) fn unknown_heatmap_session(handle: u32) -> JsValue {
    JsValue::from_str(&format!("unknown heatmap session handle {handle}"))
}

pub(crate) fn to_js_value<T>(value: &T) -> Result<JsValue, JsValue>
where
    T: Serialize,
{
    value
        .serialize(&Serializer::json_compatible())
        .map_err(js_error)
}

pub(crate) fn to_network_badge_hit(hit: NetworkPickHit) -> NetworkBadgeHit {
    let kind = match hit.kind {
        NetworkPickKind::Node => "node",
        NetworkPickKind::Toggle => "toggle",
    };
    NetworkBadgeHit {
        kind: kind.to_string(),
        node_id: hit.node_id,
    }
}

pub(crate) fn canvas_by_id(canvas_id: &str) -> Result<HtmlCanvasElement, JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("window is not available"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document is not available"))?;
    let element = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str(&format!("canvas element '{canvas_id}' was not found")))?;

    element
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str(&format!("element '{canvas_id}' is not an HTML canvas")))
}

pub(crate) fn drawing_area(
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
    let backend = CanvasBackend::with_canvas_object(canvas)
        .ok_or_else(|| JsValue::from_str("failed to create CanvasBackend from canvas element"))?;
    Ok(backend.into_drawing_area())
}

pub(crate) fn canvas_2d_context(canvas_id: &str) -> Result<CanvasRenderingContext2d, JsValue> {
    let canvas = canvas_by_id(canvas_id)?;
    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| js_error("2d canvas context is not available"))?;
    context
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| js_error("failed to cast 2d canvas context"))
}

pub(crate) async fn register_graph_image(key: String, src: String) -> Result<(), JsValue> {
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

pub(crate) fn has_graph_image(key: &str) -> bool {
    GRAPH_IMAGES.with(|images| images.borrow().contains_key(key))
}

pub(crate) fn unregister_graph_image(key: &str) -> bool {
    GRAPH_IMAGES.with(|images| images.borrow_mut().remove(key).is_some())
}

pub(crate) fn clear_graph_images() {
    GRAPH_IMAGES.with(|images| images.borrow_mut().clear());
}

pub(crate) fn clip_node_shape(
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

pub(crate) fn draw_graph_image(
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

pub(crate) fn overlay_graph_images(
    canvas_id: &str,
    nodes: &[GraphNodeRenderInfo],
) -> Result<(), JsValue> {
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

pub(crate) fn with_scatter_session_mut<T>(
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

pub(crate) fn with_tree_session_mut<T>(
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

pub(crate) fn with_bar_session_mut<T>(
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

pub(crate) fn with_heatmap_session_mut<T>(
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

pub(crate) fn create_scatter_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
    let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
    let session = ScatterSession::new(spec).map_err(plot_error_to_js)?;
    let _ = canvas_by_id(canvas_id)?;

    SCATTER_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
}

pub(crate) fn render_scatter_session(handle: u32) -> Result<(), JsValue> {
    with_scatter_session_mut(handle, |entry| {
        let root = drawing_area(
            &entry.canvas_id,
            entry.session.width(),
            entry.session.height(),
        )?;
        entry.session.render_on(root).map_err(plot_error_to_js)
    })
}

pub(crate) fn pan_scatter_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
    with_scatter_session_mut(handle, |entry| {
        entry.session.pan(delta_x, delta_y);
        Ok(())
    })
}

pub(crate) fn zoom_scatter_session(
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

pub(crate) fn pick_scatter_point_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    with_scatter_session_mut(handle, |entry| {
        let hit = entry.session.pick_point(canvas_x, canvas_y);
        to_js_value(&hit.map(|index| ScatterHit { index }))
    })
}

pub(crate) fn set_scatter_selection(handle: u32, index: Option<u32>) -> Result<(), JsValue> {
    with_scatter_session_mut(handle, |entry| {
        entry
            .session
            .set_selection(index.map(|value| value as usize));
        Ok(())
    })
}

pub(crate) fn destroy_scatter_session(handle: u32) -> Result<(), JsValue> {
    SCATTER_SESSIONS.with(|store| {
        let removed = store.borrow_mut().sessions.remove(&handle);
        if removed.is_some() {
            Ok(())
        } else {
            Err(unknown_scatter_session(handle))
        }
    })
}

pub(crate) fn render_scatter(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
    let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
    let root = drawing_area(canvas_id, spec.width, spec.height)?;
    render_scatter_on(root, &spec).map_err(plot_error_to_js)
}

pub(crate) fn render_tree(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
    let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
    let root = drawing_area(canvas_id, spec.width, spec.height)?;
    render_tree_on(root, &spec).map_err(plot_error_to_js)?;
    let nodes = tree_render_nodes(&spec).map_err(plot_error_to_js)?;
    overlay_graph_images(canvas_id, &nodes)
}

pub(crate) fn create_tree_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
    let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
    let session = TreeSession::new(spec).map_err(plot_error_to_js)?;
    let _ = canvas_by_id(canvas_id)?;
    TREE_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
}

pub(crate) fn render_tree_session(handle: u32) -> Result<(), JsValue> {
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

pub(crate) fn render_tree_session_transition(handle: u32, progress: f64) -> Result<(), JsValue> {
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

pub(crate) fn pan_tree_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
    with_tree_session_mut(handle, |entry| {
        entry.session.pan(delta_x, delta_y);
        Ok(())
    })
}

pub(crate) fn zoom_tree_session(
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

pub(crate) fn pick_tree_node_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    with_tree_session_mut(handle, |entry| {
        let hit = entry.session.pick_node(canvas_x, canvas_y);
        to_js_value(&hit.map(|node_id| TreeHit { node_id }))
    })
}

pub(crate) fn set_tree_selection(handle: u32, node_id: Option<String>) -> Result<(), JsValue> {
    with_tree_session_mut(handle, |entry| {
        entry.session.set_selection(node_id);
        Ok(())
    })
}

pub(crate) fn toggle_tree_node_collapsed_session(
    handle: u32,
    node_id: String,
) -> Result<bool, JsValue> {
    with_tree_session_mut(handle, |entry| Ok(entry.session.toggle_collapse(&node_id)))
}

pub(crate) fn set_tree_node_collapsed_session(
    handle: u32,
    node_id: String,
    collapsed: bool,
) -> Result<(), JsValue> {
    with_tree_session_mut(handle, |entry| {
        entry.session.set_collapsed(&node_id, collapsed);
        Ok(())
    })
}

pub(crate) fn destroy_tree_session(handle: u32) -> Result<(), JsValue> {
    TREE_SESSIONS.with(|store| {
        store
            .borrow_mut()
            .sessions
            .remove(&handle)
            .map(|_| ())
            .ok_or_else(|| unknown_tree_session(handle))
    })
}

pub(crate) fn pan_scatter(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
    let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
    let next = pan_scatter_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
    to_js_value(&next)
}

pub(crate) fn pan_tree(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
    let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
    let next = pan_tree_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
    to_js_value(&next)
}

pub(crate) fn pick_scatter_point(
    spec: JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    let spec: ScatterPlotSpec = from_value(spec).map_err(js_error)?;
    let hit = core_pick_scatter_point(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
    to_js_value(&hit.map(|index| ScatterHit { index }))
}

pub(crate) fn pick_tree_node(
    spec: JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    let spec: TreePlotSpec = from_value(spec).map_err(js_error)?;
    let hit = core_pick_tree_node(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
    to_js_value(&hit.map(|node_id| TreeHit { node_id }))
}
