use super::*;

#[derive(Debug)]
pub struct NetworkSession {
    pub(super) spec: NetworkPlotSpec,
    pub(super) layout: BTreeMap<String, (f64, f64)>,
    pub(super) resolved: BTreeMap<String, ResolvedNode>,
    pub(super) selection_style: ResolvedSelectionStyle,
    pub(super) view: ScreenTransform,
    pub(super) transition: Option<NetworkTransition>,
    pub(super) tracking: Option<NetworkTrackedPath>,
}

impl NetworkSession {
    pub fn new(spec: NetworkPlotSpec) -> Result<Self, PlotError> {
        validate(&spec)?;
        let resolved = resolve_nodes(&spec)?;
        let layout = compute_layout_with_zoom(&spec, 1.0)?;
        let selection_style =
            resolve_selection_style(SELECTION_RING_PADDING, spec.selection_style.as_ref())?;
        let view = ScreenTransform::new(spec.offset_x as f64, spec.offset_y as f64);
        Ok(Self {
            spec,
            layout,
            resolved,
            selection_style,
            view,
            transition: None,
            tracking: None,
        })
    }

    pub fn update_spec(&mut self, spec: NetworkPlotSpec) -> Result<(), PlotError> {
        validate(&spec)?;
        let topology_changed = topology_changed(&self.spec, &spec);
        let previous_spec = self.spec.clone();
        let previous_layout = self.layout.clone();
        let previous_resolved = self.resolved.clone();
        let previous_selected_node_id = self.spec.selected_node_id.clone();
        let resolved = resolve_nodes(&spec)?;
        let layout = compute_layout_from_previous_with_zoom(
            &self.layout,
            &self.spec,
            &spec,
            self.view.zoom,
        )?;
        let selection_style =
            resolve_selection_style(SELECTION_RING_PADDING, spec.selection_style.as_ref())?;
        let view = self.view;
        let tracking = self.tracking.as_ref().and_then(|tracking| {
            NetworkTrackedPath::resolve(&spec, tracking.node_ids.clone())
                .ok()
                .flatten()
                .map(|mut resolved_tracking| {
                    resolved_tracking.set_progress(tracking.progress);
                    resolved_tracking.set_breath_phase(tracking.breath_phase);
                    resolved_tracking
                })
        });
        let anchor_node_id = topology_changed
            .then(|| choose_transition_anchor(&previous_spec, &spec, &previous_layout, &layout));
        self.spec = spec;
        self.layout = layout;
        self.resolved = resolved;
        self.selection_style = selection_style;
        self.view = view;
        self.tracking = tracking;
        self.transition = topology_changed.then(|| NetworkTransition {
            from_spec: previous_spec,
            from_layout: previous_layout,
            from_resolved: previous_resolved,
            from_selected_node_id: previous_selected_node_id,
            anchor_node_id: anchor_node_id.expect("anchor present for topology transition"),
        });
        self.spec.selected_node_id = self
            .spec
            .selected_node_id
            .clone()
            .filter(|node_id| self.layout.contains_key(node_id.as_str()));
        self.sync_view_to_spec();
        Ok(())
    }

    pub fn render_on<DB>(&self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        render_with_layout(
            &root,
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
            self.tracking.as_ref(),
        )
    }

    pub fn render_transition_on<DB>(
        &self,
        root: PlotArea<DB>,
        progress: f64,
    ) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        let Some(transition) = self.transition.as_ref() else {
            return self.render_on(root);
        };
        render_transition_with_layout(
            &root,
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
            transition,
            progress,
        )
    }

    pub fn pick(&self, canvas_x: f64, canvas_y: f64) -> Option<NetworkPickHit> {
        pick_hit_from_layout(
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
            canvas_x,
            canvas_y,
        )
    }

    pub fn pick_node(&self, canvas_x: f64, canvas_y: f64) -> Option<String> {
        self.pick(canvas_x, canvas_y)
            .and_then(|hit| (hit.kind == NetworkPickKind::Node).then_some(hit.node_id))
    }

    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        self.view.pan_by(delta_x, delta_y);
        self.sync_view_to_spec();
    }

    pub fn zoom_at(&mut self, canvas_x: f64, canvas_y: f64, factor: f64) -> Result<(), PlotError> {
        self.view.zoom_at(canvas_x, canvas_y, factor)?;
        self.sync_view_to_spec();
        Ok(())
    }

    pub fn set_selection(&mut self, node_id: Option<String>) {
        self.spec.selected_node_id = node_id.filter(|id| self.layout.contains_key(id.as_str()));
    }

    pub fn set_tracking_path(&mut self, node_ids: Vec<String>) -> Result<(), PlotError> {
        self.tracking = NetworkTrackedPath::resolve(&self.spec, node_ids)?;
        Ok(())
    }

    pub fn set_tracking_progress(&mut self, progress: f64) {
        if let Some(tracking) = self.tracking.as_mut() {
            tracking.set_progress(progress);
        }
    }

    pub fn set_tracking_breath_phase(&mut self, breath_phase: f64) {
        if let Some(tracking) = self.tracking.as_mut() {
            tracking.set_breath_phase(breath_phase);
        }
    }

    pub fn clear_tracking_path(&mut self) {
        self.tracking = None;
    }

    pub fn view(&self) -> NetworkView {
        NetworkView {
            zoom: self.view.zoom,
            translate_x: self.view.translate_x,
            translate_y: self.view.translate_y,
        }
    }

    pub fn set_view(&mut self, view: NetworkView) -> Result<(), PlotError> {
        self.view = ScreenTransform::with_view(view.zoom, view.translate_x, view.translate_y)?;
        self.sync_view_to_spec();
        Ok(())
    }

    pub fn compute_focus_view(
        &self,
        node_id: &str,
        options: Option<NetworkFocusOptions>,
    ) -> Option<NetworkView> {
        let center = self.layout.get(node_id).copied()?;
        let options = options.unwrap_or_default();
        let padding = options.padding.max(0.0);
        let min_world_span = options.min_world_span.max(1.0);
        let mut focused_ids = HashSet::from([node_id.to_string()]);
        match options.mode {
            NetworkFocusMode::NodeAndNeighbors => {
                for neighbor_id in neighbor_ids(&self.spec, node_id) {
                    focused_ids.insert(neighbor_id);
                }
            }
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for focused_id in &focused_ids {
            let Some(&(x, y)) = self.layout.get(focused_id) else {
                continue;
            };
            let radius = self
                .resolved
                .get(focused_id)
                .map(|node| node.style.radius.max(1) as f64)
                .unwrap_or(self.spec.node_radius.max(1) as f64);
            min_x = min_x.min(x - radius);
            min_y = min_y.min(y - radius);
            max_x = max_x.max(x + radius);
            max_y = max_y.max(y + radius);
        }

        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            min_x = center.0 - min_world_span / 2.0;
            max_x = center.0 + min_world_span / 2.0;
            min_y = center.1 - min_world_span / 2.0;
            max_y = center.1 + min_world_span / 2.0;
        }

        let span_x = (max_x - min_x).max(min_world_span);
        let span_y = (max_y - min_y).max(min_world_span);
        let available_width = (self.spec.width as f64 - padding * 2.0).max(1.0);
        let available_height = (self.spec.height as f64 - padding * 2.0).max(1.0);
        let zoom = (available_width / span_x).min(available_height / span_y);
        let view = NetworkView {
            zoom,
            translate_x: self.spec.width as f64 / 2.0 - ((min_x + max_x) / 2.0) * zoom,
            translate_y: self.spec.height as f64 / 2.0 - ((min_y + max_y) / 2.0) * zoom,
        };
        ScreenTransform::with_view(view.zoom, view.translate_x, view.translate_y)
            .ok()
            .map(|clamped| NetworkView {
                zoom: clamped.zoom,
                translate_x: clamped.translate_x,
                translate_y: clamped.translate_y,
            })
    }

    pub fn spec(&self) -> &NetworkPlotSpec {
        &self.spec
    }

    pub fn into_spec(self) -> NetworkPlotSpec {
        self.spec
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }

    pub fn render_nodes(&self) -> Vec<GraphNodeRenderInfo> {
        render_nodes_with_layout(
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
        )
    }

    pub fn render_transition_nodes(&self, progress: f64) -> Vec<GraphNodeRenderInfo> {
        let Some(transition) = self.transition.as_ref() else {
            return self.render_nodes();
        };
        render_transition_nodes_with_layout(
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
            transition,
            progress,
        )
    }

    pub fn has_transition(&self) -> bool {
        self.transition.is_some()
    }

    pub fn clear_transition(&mut self) {
        self.transition = None;
    }

    fn sync_view_to_spec(&mut self) {
        self.spec.offset_x = self.view.translate_x.round() as i32;
        self.spec.offset_y = self.view.translate_y.round() as i32;
    }
}
