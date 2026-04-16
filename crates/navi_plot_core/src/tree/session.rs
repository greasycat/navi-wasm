use super::*;

#[derive(Debug, Clone)]
pub struct TreeSession {
    pub(super) spec: TreePlotSpec,
    pub(super) validated: ValidatedTree,
    pub(super) visible: VisibleTree,
    pub(super) layout: BTreeMap<String, LayoutPoint>,
    pub(super) resolved_nodes: BTreeMap<String, ResolvedTreeNode>,
    pub(super) selection_style: crate::graph_style::ResolvedSelectionStyle,
    pub(super) view: ScreenTransform,
    pub(super) transition: Option<TreeTransition>,
}

impl TreeSession {
    pub fn new(spec: TreePlotSpec) -> Result<Self, PlotError> {
        ensure_dimensions(spec.width, spec.height)?;
        let validated = validate_tree(&spec)?;
        let resolved_nodes = resolve_tree_nodes(&spec)?;
        let selection_style =
            resolve_selection_style(SELECTION_RING_PADDING, spec.selection_style.as_ref())?;
        let view = ScreenTransform::new(spec.offset_x as f64, spec.offset_y as f64);
        let mut session = Self {
            spec,
            validated,
            visible: VisibleTree::default(),
            layout: BTreeMap::new(),
            resolved_nodes,
            selection_style,
            view,
            transition: None,
        };
        session.refresh_visible_state(None);
        Ok(session)
    }

    pub fn render_on<DB>(&self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        render_with_layout(
            &root,
            &self.spec,
            &self.visible,
            &self.layout,
            &self.resolved_nodes,
            &self.selection_style,
            &self.view,
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

        let progress = progress.clamp(0.0, 1.0);
        root.fill(&WHITE).map_err(backend_error)?;
        draw_tree_title(&root, &self.spec)?;
        let viewport = PixelBounds::from_canvas(self.spec.width, self.spec.height);
        let anchor_frame = transition_anchor_frame(transition, &self.layout, progress);

        for edge in &self.spec.edges {
            let edge_style = resolve_tree_edge_style(&self.spec, edge)?;
            let Some(source_state) = transition_node_frame(
                &edge.source,
                transition,
                &self.layout,
                anchor_frame,
                progress,
            ) else {
                continue;
            };
            let Some(target_state) = transition_node_frame(
                &edge.target,
                transition,
                &self.layout,
                anchor_frame,
                progress,
            ) else {
                continue;
            };
            let edge_alpha = source_state
                .opacity
                .min(target_state.opacity)
                .clamp(0.0, 1.0);
            if edge_alpha <= 0.0 || edge_style.stroke_width == 0 {
                continue;
            }

            let source = project_point_f64(source_state.point, &self.view);
            let target = project_point_f64(target_state.point, &self.view);
            let Some((clipped_source, clipped_target)) = viewport.clip_line(source, target) else {
                continue;
            };

            let shape_style =
                ShapeStyle::from(&edge_style.stroke_color.mix(edge_style.opacity * edge_alpha))
                    .stroke_width(edge_style.stroke_width);
            for (segment_start, segment_end) in edge_line_segments(
                clipped_source,
                clipped_target,
                edge_style.dash_pattern.as_deref(),
            ) {
                root.draw(&PathElement::new(
                    vec![segment_start, segment_end],
                    shape_style,
                ))
                .map_err(backend_error)?;
            }
        }

        for node in &self.spec.nodes {
            let Some(node_state) =
                transition_node_frame(&node.id, transition, &self.layout, anchor_frame, progress)
            else {
                continue;
            };
            if node_state.opacity <= 0.0 {
                continue;
            }

            let position = project_point_f64(node_state.point, &self.view);
            let resolved_node = resolved_tree_node(&self.resolved_nodes, &node.id)?;
            let scaled_style = scale_node_style(&resolved_node.style, self.view.zoom);
            let scaled_selection_style =
                scale_selection_style(&self.selection_style, self.view.zoom);
            let selection_alpha = transition_phase_opacity(
                transition.from_selected_node_id.as_deref() == Some(node.id.as_str()),
                self.spec.selected_node_id.as_deref() == Some(node.id.as_str()),
                progress,
            );
            if !node_intersects_viewport(
                viewport,
                position,
                &scaled_style,
                &scaled_selection_style,
                selection_alpha > 0.0,
            ) {
                continue;
            }

            draw_tree_node(
                &root,
                position,
                &scaled_style,
                resolved_node.media.as_ref(),
                &node.label,
                selection_alpha > 0.0,
                &scaled_selection_style,
                self.spec.pixel_ratio,
                node_state.opacity,
                selection_alpha,
            )?;

            let marker_alpha = transition_phase_opacity(
                transition
                    .from_visible
                    .collapsed_marker_node_ids
                    .contains(&node.id),
                self.visible.collapsed_marker_node_ids.contains(&node.id),
                progress,
            );
            if marker_alpha > 0.0 {
                draw_collapsed_marker(&root, position, &scaled_style, marker_alpha)?;
            }
        }

        root.present().map_err(backend_error)?;
        Ok(())
    }

    pub fn pick_node(&self, canvas_x: f64, canvas_y: f64) -> Option<String> {
        pick_from_layout(
            &self.spec,
            &self.layout,
            &self.resolved_nodes,
            &self.selection_style,
            &self.view,
            canvas_x,
            canvas_y,
        )
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

    pub fn toggle_collapse(&mut self, node_id: &str) -> bool {
        if !self.validated.has_children(node_id) {
            return false;
        }

        let collapsed_node_ids = normalized_collapsed_node_ids(&self.spec, &self.validated);
        let next = !collapsed_node_ids.contains(node_id);
        self.set_collapsed(node_id, next);
        next
    }

    pub fn set_collapsed(&mut self, node_id: &str, collapsed: bool) {
        if !self.validated.has_children(node_id) {
            return;
        }

        let mut collapsed_node_ids = normalized_collapsed_node_ids(&self.spec, &self.validated);
        let was_collapsed = collapsed_node_ids.contains(node_id);
        if was_collapsed == collapsed {
            return;
        }
        let from_layout = self.layout.clone();
        let from_visible = self.visible.clone();
        let from_selected_node_id = self.spec.selected_node_id.clone();
        if collapsed {
            collapsed_node_ids.insert(node_id.to_string());
        } else {
            collapsed_node_ids.remove(node_id);
        }
        sync_collapsed_node_ids(&mut self.spec, &collapsed_node_ids);
        self.refresh_visible_state(collapsed.then_some(node_id));
        self.transition = Some(TreeTransition {
            from_layout,
            from_visible,
            from_selected_node_id,
            anchor_node_id: node_id.to_string(),
        });
    }

    pub fn spec(&self) -> &TreePlotSpec {
        &self.spec
    }

    pub fn into_spec(self) -> TreePlotSpec {
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
            &self.resolved_nodes,
            &self.selection_style,
            &self.view,
        )
    }

    pub fn render_transition_nodes(&self, progress: f64) -> Vec<GraphNodeRenderInfo> {
        let Some(transition) = self.transition.as_ref() else {
            return self.render_nodes();
        };

        let progress = progress.clamp(0.0, 1.0);
        let anchor_frame = transition_anchor_frame(transition, &self.layout, progress);
        let viewport = PixelBounds::from_canvas(self.spec.width, self.spec.height);

        self.spec
            .nodes
            .iter()
            .filter_map(|node| {
                let node_state = transition_node_frame(
                    &node.id,
                    transition,
                    &self.layout,
                    anchor_frame,
                    progress,
                )?;
                if node_state.opacity <= 0.0 {
                    return None;
                }
                let position = project_point_f64(node_state.point, &self.view);
                let resolved_node = self.resolved_nodes.get(&node.id)?;
                let scaled_style = scale_node_style(&resolved_node.style, self.view.zoom);
                let scaled_selection_style =
                    scale_selection_style(&self.selection_style, self.view.zoom);
                let selection_alpha = transition_phase_opacity(
                    transition.from_selected_node_id.as_deref() == Some(node.id.as_str()),
                    self.spec.selected_node_id.as_deref() == Some(node.id.as_str()),
                    progress,
                );
                if !node_intersects_viewport(
                    viewport,
                    position,
                    &scaled_style,
                    &scaled_selection_style,
                    selection_alpha > 0.0,
                ) {
                    return None;
                }
                Some(GraphNodeRenderInfo {
                    id: node.id.clone(),
                    center_x: position.0,
                    center_y: position.1,
                    radius: scaled_style.radius,
                    shape: scaled_style.shape.clone(),
                    opacity: scaled_style.opacity * node_state.opacity,
                    media: resolved_node.media.clone(),
                })
            })
            .collect()
    }

    fn sync_view_to_spec(&mut self) {
        self.spec.offset_x = self.view.translate_x.round() as i32;
        self.spec.offset_y = self.view.translate_y.round() as i32;
    }

    fn refresh_visible_state(&mut self, preferred_selection: Option<&str>) {
        let collapsed_node_ids = normalized_collapsed_node_ids(&self.spec, &self.validated);
        sync_collapsed_node_ids(&mut self.spec, &collapsed_node_ids);
        self.visible = build_visible_tree(&self.spec.root_id, &self.validated, &collapsed_node_ids);
        self.layout = build_layout(&self.spec, &self.visible.children_by_parent);
        sync_selected_node_id(
            &mut self.spec.selected_node_id,
            &self.layout,
            &self.validated,
            preferred_selection,
        );
    }
}
