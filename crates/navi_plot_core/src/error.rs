use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum PlotError {
    #[error("plot width and height must be greater than zero, got {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("scatter plot requires at least one point")]
    EmptyScatterData,
    #[error("tree plot requires at least one node")]
    EmptyTree,
    #[error("scatter point `{axis}` value must be finite, got {value}")]
    NonFinitePointValue { axis: &'static str, value: f64 },
    #[error("invalid {axis}-axis range: min {min} must be less than max {max}")]
    InvalidRange {
        axis: &'static str,
        min: f64,
        max: f64,
    },
    #[error("invalid color `{color}`")]
    InvalidColor { color: String },
    #[error("invalid style field `{field}`: {reason} (got {value})")]
    InvalidStyleValue {
        field: &'static str,
        value: f64,
        reason: &'static str,
    },
    #[error("invalid zoom factor `{factor}`: must be finite and greater than zero")]
    InvalidZoomFactor { factor: f64 },
    #[error("invalid node media field `{field}`: {reason}")]
    InvalidNodeMedia {
        field: &'static str,
        reason: &'static str,
    },
    #[error("tree root `{root_id}` was not found in the node set")]
    MissingRoot { root_id: String },
    #[error("duplicate tree node id `{node_id}`")]
    DuplicateNodeId { node_id: String },
    #[error("duplicate tree edge `{from_node}` -> `{to_node}`")]
    DuplicateEdge { from_node: String, to_node: String },
    #[error("tree edge references unknown node `{node_id}`")]
    UnknownNode { node_id: String },
    #[error("tree must contain exactly one root, found {count}")]
    InvalidRootCount { count: usize },
    #[error("declared root `{declared_root}` does not match the unique root `{actual_root}`")]
    RootMismatch {
        declared_root: String,
        actual_root: String,
    },
    #[error("tree node `{node_id}` has {parent_count} parents, expected exactly one")]
    InvalidParentCount {
        node_id: String,
        parent_count: usize,
    },
    #[error("tree contains a cycle")]
    CycleDetected,
    #[error("tree node `{node_id}` is not reachable from root `{root_id}`")]
    DisconnectedNode { node_id: String, root_id: String },
    #[error("rendering backend error: {message}")]
    Backend { message: String },

    // Line chart errors
    #[error("line chart requires at least one series")]
    EmptyLineSeries,
    #[error("line series {series_index} has no points")]
    EmptySeriesPoints { series_index: usize },

    // Bar chart errors
    #[error("bar chart requires at least one category")]
    EmptyBarCategories,
    #[error("bar chart requires at least one series")]
    EmptyBarSeries,
    #[error("bar series {series_index} has {actual} values but expected {expected}")]
    BarValueCountMismatch {
        series_index: usize,
        expected: usize,
        actual: usize,
    },
    #[error("stacked bar chart does not support negative values (series {series_index}, category {category_index})")]
    NegativeStackedBarValue {
        series_index: usize,
        category_index: usize,
    },

    // Heatmap errors
    #[error("heatmap requires at least one cell")]
    EmptyHeatmapData,
    #[error("heatmap row {row_index} has {actual_cols} columns but expected {expected_cols}")]
    HeatmapShapeMismatch {
        expected_cols: usize,
        row_index: usize,
        actual_cols: usize,
    },

    // Network errors
    #[error("network plot requires at least one node")]
    EmptyNetwork,
}

impl PlotError {
    /// Machine-readable error code in `SCREAMING_SNAKE_CASE`.
    ///
    /// Included as the `.code` property on thrown JS `Error` objects so callers
    /// can distinguish error types programmatically:
    ///
    /// ```js
    /// try { wasm.render_scatter(id, spec) }
    /// catch (e) { if (e.code === "EMPTY_SCATTER_DATA") { ... } }
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidDimensions { .. } => "INVALID_DIMENSIONS",
            Self::EmptyScatterData => "EMPTY_SCATTER_DATA",
            Self::EmptyTree => "EMPTY_TREE",
            Self::NonFinitePointValue { .. } => "NON_FINITE_POINT_VALUE",
            Self::InvalidRange { .. } => "INVALID_RANGE",
            Self::InvalidColor { .. } => "INVALID_COLOR",
            Self::InvalidStyleValue { .. } => "INVALID_STYLE_VALUE",
            Self::InvalidZoomFactor { .. } => "INVALID_ZOOM_FACTOR",
            Self::InvalidNodeMedia { .. } => "INVALID_NODE_MEDIA",
            Self::MissingRoot { .. } => "MISSING_ROOT",
            Self::DuplicateNodeId { .. } => "DUPLICATE_NODE_ID",
            Self::DuplicateEdge { .. } => "DUPLICATE_EDGE",
            Self::UnknownNode { .. } => "UNKNOWN_NODE",
            Self::InvalidRootCount { .. } => "INVALID_ROOT_COUNT",
            Self::RootMismatch { .. } => "ROOT_MISMATCH",
            Self::InvalidParentCount { .. } => "INVALID_PARENT_COUNT",
            Self::CycleDetected => "CYCLE_DETECTED",
            Self::DisconnectedNode { .. } => "DISCONNECTED_NODE",
            Self::Backend { .. } => "BACKEND_ERROR",
            Self::EmptyLineSeries => "EMPTY_LINE_SERIES",
            Self::EmptySeriesPoints { .. } => "EMPTY_SERIES_POINTS",
            Self::EmptyBarCategories => "EMPTY_BAR_CATEGORIES",
            Self::EmptyBarSeries => "EMPTY_BAR_SERIES",
            Self::BarValueCountMismatch { .. } => "BAR_VALUE_COUNT_MISMATCH",
            Self::NegativeStackedBarValue { .. } => "NEGATIVE_STACKED_BAR_VALUE",
            Self::EmptyHeatmapData => "EMPTY_HEATMAP_DATA",
            Self::HeatmapShapeMismatch { .. } => "HEATMAP_SHAPE_MISMATCH",
            Self::EmptyNetwork => "EMPTY_NETWORK",
        }
    }
}
