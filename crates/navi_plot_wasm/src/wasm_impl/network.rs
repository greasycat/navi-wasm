use super::*;

// ── Network session store ────────────────────────────────────────────────

pub(crate) struct NetworkCanvasSession {
    canvas_id: String,
    session: NetworkSession,
}

pub(crate) struct NetworkSessionStore {
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

pub(crate) fn unknown_network_session(handle: u32) -> JsValue {
    JsValue::from_str(&format!("unknown network session handle {handle}"))
}

pub(crate) fn with_network_session_mut<T>(
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

pub(crate) fn create_network_session(canvas_id: &str, spec: JsValue) -> Result<u32, JsValue> {
    let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
    let spec = normalize_network_spec_for_canvas(canvas_id, spec)?;
    let session = NetworkSession::new(spec).map_err(plot_error_to_js)?;
    let _ = canvas_by_id(canvas_id)?;
    NETWORK_SESSIONS.with(|store| Ok(store.borrow_mut().insert(canvas_id.to_string(), session)))
}

pub(crate) fn update_network_session(handle: u32, spec: JsValue) -> Result<(), JsValue> {
    let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
    with_network_session_mut(handle, |entry| {
        let spec = normalize_network_spec_for_canvas(&entry.canvas_id, spec)?;
        entry.session.update_spec(spec).map_err(plot_error_to_js)
    })
}

pub(crate) fn render_network_session(handle: u32) -> Result<(), JsValue> {
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

pub(crate) fn render_network_transition_session(handle: u32, progress: f64) -> Result<(), JsValue> {
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

pub(crate) fn pan_network_session(handle: u32, delta_x: f64, delta_y: f64) -> Result<(), JsValue> {
    with_network_session_mut(handle, |entry| {
        entry.session.pan(delta_x, delta_y);
        Ok(())
    })
}

pub(crate) fn zoom_network_session(
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

pub(crate) fn pick_network_node_session(
    handle: u32,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    with_network_session_mut(handle, |entry| {
        let hit = entry.session.pick_node(canvas_x, canvas_y);
        to_js_value(&hit.map(|node_id| NetworkHit { node_id }))
    })
}

pub(crate) fn pick_network_hit_session(
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

pub(crate) fn set_network_selection(handle: u32, node_id: Option<String>) -> Result<(), JsValue> {
    with_network_session_mut(handle, |entry| {
        entry.session.set_selection(node_id.clone());
        Ok(())
    })
}

pub(crate) fn set_network_tracking_path_session(
    handle: u32,
    node_ids: JsValue,
) -> Result<(), JsValue> {
    let node_ids: Vec<String> = from_value(node_ids).map_err(js_error)?;
    with_network_session_mut(handle, |entry| {
        entry
            .session
            .set_tracking_path(node_ids)
            .map_err(plot_error_to_js)
    })
}

pub(crate) fn set_network_tracking_progress_session(
    handle: u32,
    progress: f64,
) -> Result<(), JsValue> {
    with_network_session_mut(handle, |entry| {
        entry.session.set_tracking_progress(progress);
        Ok(())
    })
}

pub(crate) fn set_network_tracking_breath_phase_session(
    handle: u32,
    phase: f64,
) -> Result<(), JsValue> {
    with_network_session_mut(handle, |entry| {
        entry.session.set_tracking_breath_phase(phase);
        Ok(())
    })
}

pub(crate) fn clear_network_tracking_path_session(handle: u32) -> Result<(), JsValue> {
    with_network_session_mut(handle, |entry| {
        entry.session.clear_tracking_path();
        Ok(())
    })
}

pub(crate) fn get_network_view_session(handle: u32) -> Result<JsValue, JsValue> {
    with_network_session_mut(handle, |entry| to_js_value(&entry.session.view()))
}

pub(crate) fn set_network_view_session(handle: u32, view: JsValue) -> Result<(), JsValue> {
    let view: NetworkView = from_value(view).map_err(js_error)?;
    with_network_session_mut(handle, |entry| {
        entry.session.set_view(view).map_err(plot_error_to_js)
    })
}

pub(crate) fn compute_network_focus_view_session(
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

pub(crate) fn has_network_transition_session(handle: u32) -> Result<bool, JsValue> {
    with_network_session_mut(handle, |entry| Ok(entry.session.has_transition()))
}

pub(crate) fn clear_network_transition_session(handle: u32) -> Result<(), JsValue> {
    with_network_session_mut(handle, |entry| {
        entry.session.clear_transition();
        Ok(())
    })
}

pub(crate) fn destroy_network_session(handle: u32) -> Result<(), JsValue> {
    NETWORK_SESSIONS.with(|store| {
        store
            .borrow_mut()
            .sessions
            .remove(&handle)
            .map(|_| ())
            .ok_or_else(|| unknown_network_session(handle))
    })
}

pub(crate) fn render_network(canvas_id: &str, spec: JsValue) -> Result<(), JsValue> {
    let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
    let spec = normalize_network_spec_for_canvas(canvas_id, spec)?;
    let root = drawing_area(canvas_id, spec.width, spec.height)?;
    render_network_on(root, &spec).map_err(plot_error_to_js)?;
    let nodes = network_render_nodes(&spec).map_err(plot_error_to_js)?;
    overlay_graph_images(canvas_id, &nodes)
}

pub(crate) fn pan_network(spec: JsValue, delta_x: f64, delta_y: f64) -> Result<JsValue, JsValue> {
    let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
    let next = pan_network_spec(&spec, delta_x, delta_y).map_err(plot_error_to_js)?;
    to_js_value(&next)
}

pub(crate) fn pick_network_node(
    spec: JsValue,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<JsValue, JsValue> {
    let spec: NetworkPlotSpec = from_value(spec).map_err(js_error)?;
    let hit = core_pick_network_node(&spec, canvas_x, canvas_y).map_err(plot_error_to_js)?;
    to_js_value(&hit.map(|node_id| NetworkHit { node_id }))
}

pub(crate) fn pick_network_hit(
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
