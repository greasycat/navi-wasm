use super::*;

// ── Line chart session store ─────────────────────────────────────────────

pub(crate) struct LineCanvasSession {
    canvas_id: String,
    session: LineSession,
}

pub(crate) struct LineSessionStore {
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

pub(crate) fn unknown_line_session(handle: u32) -> JsValue {
    JsValue::from_str(&format!("unknown line session handle {handle}"))
}

pub(crate) fn with_line_session_mut<T>(
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

pub(crate) fn create_line_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
    let spec: LinePlotSpec = from_value(spec).map_err(js_error)?;
    let session = LineSession::new(spec).map_err(plot_error_to_js)?;
    let _ = canvas_by_id(canvas_id)?;
    LINE_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
}

pub(crate) fn render_line_session(handle: u32) -> Result<(), JsValue> {
    with_line_session_mut(handle, |entry| {
        let root = drawing_area(
            &entry.canvas_id,
            entry.session.width(),
            entry.session.height(),
        )?;
        entry.session.render_on(root).map_err(plot_error_to_js)
    })
}

pub(crate) fn pan_line_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
    with_line_session_mut(handle, |entry| {
        entry.session.pan(delta_x, delta_y);
        Ok(())
    })
}

pub(crate) fn zoom_line_session(
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

pub(crate) fn pick_line_point_session(
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

pub(crate) fn set_line_selection(
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

pub(crate) fn destroy_line_session(handle: u32) -> Result<(), JsValue> {
    LINE_SESSIONS.with(|store| {
        store
            .borrow_mut()
            .sessions
            .remove(&handle)
            .map(|_| ())
            .ok_or_else(|| unknown_line_session(handle))
    })
}

pub(crate) fn render_line(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
    let spec: LinePlotSpec = from_value(spec).map_err(js_error)?;
    let root = drawing_area(canvas_id, spec.width, spec.height)?;
    render_line_on(root, &spec).map_err(plot_error_to_js)
}

pub(crate) fn pan_line(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
    let spec: LinePlotSpec = from_value(spec).map_err(js_error)?;
    let next = pan_line_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
    to_js_value(&next)
}

pub(crate) fn pick_line_point(
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

pub(crate) fn create_bar_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
    let spec: BarChartSpec = from_value(spec).map_err(js_error)?;
    let session = BarSession::new(spec).map_err(plot_error_to_js)?;
    let _ = canvas_by_id(canvas_id)?;
    BAR_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
}

pub(crate) fn render_bar_session(handle: u32) -> Result<(), JsValue> {
    with_bar_session_mut(handle, |entry| {
        let root = drawing_area(
            &entry.canvas_id,
            entry.session.width(),
            entry.session.height(),
        )?;
        entry.session.render_on(root).map_err(plot_error_to_js)
    })
}

pub(crate) fn zoom_bar_session(
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

pub(crate) fn pick_bar_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    with_bar_session_mut(handle, |entry| {
        let hit = entry.session.pick_bar(canvas_x, canvas_y);
        to_js_value(&hit.map(|[series_index, category_index]| BarHit {
            series_index,
            category_index,
        }))
    })
}

pub(crate) fn set_bar_selection(
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

pub(crate) fn destroy_bar_session(handle: u32) -> Result<(), JsValue> {
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

pub(crate) fn render_bar(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
    let spec: BarChartSpec = from_value(spec).map_err(js_error)?;
    let root = drawing_area(canvas_id, spec.width, spec.height)?;
    render_bar_on(root, &spec).map_err(plot_error_to_js)
}

pub(crate) fn pick_bar(spec: JsValue, canvas_x: f64, canvas_y: f64) -> Result<JsValue, JsValue> {
    let spec: BarChartSpec = from_value(spec).map_err(js_error)?;
    let hit = core_pick_bar(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
    to_js_value(&hit.map(|[si, ci]| BarHit {
        series_index: si,
        category_index: ci,
    }))
}

// ── Heatmap session store ────────────────────────────────────────────────

pub(crate) fn create_heatmap_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
    let spec: HeatmapSpec = from_value(spec).map_err(js_error)?;
    let session = HeatmapSession::new(spec).map_err(plot_error_to_js)?;
    let _ = canvas_by_id(canvas_id)?;
    HEATMAP_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
}

pub(crate) fn render_heatmap_session(handle: u32) -> Result<(), JsValue> {
    with_heatmap_session_mut(handle, |entry| {
        let root = drawing_area(
            &entry.canvas_id,
            entry.session.width(),
            entry.session.height(),
        )?;
        entry.session.render_on(root).map_err(plot_error_to_js)
    })
}

pub(crate) fn zoom_heatmap_session(
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

pub(crate) fn pick_heatmap_cell_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    with_heatmap_session_mut(handle, |entry| {
        let hit = entry.session.pick_cell(canvas_x, canvas_y);
        to_js_value(&hit.map(|[row, col]| HeatmapHit { row, col }))
    })
}

pub(crate) fn set_heatmap_selection(
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

pub(crate) fn destroy_heatmap_session(handle: u32) -> Result<(), JsValue> {
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

pub(crate) fn render_heatmap(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
    let spec: HeatmapSpec = from_value(spec).map_err(js_error)?;
    let root = drawing_area(canvas_id, spec.width, spec.height)?;
    render_heatmap_on(root, &spec).map_err(plot_error_to_js)
}

pub(crate) fn pick_heatmap_cell(
    spec: JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    let spec: HeatmapSpec = from_value(spec).map_err(js_error)?;
    let hit = core_pick_heatmap_cell(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
    to_js_value(&hit.map(|[row, col]| HeatmapHit { row, col }))
}
