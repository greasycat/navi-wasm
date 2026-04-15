#[cfg(target_arch = "wasm32")]
mod wasm_impl;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> wasm_bindgen::JsValue {
    wasm_bindgen::JsValue::from_str(
        "navi_plot_wasm is only available for the wasm32-unknown-unknown target",
    )
}

macro_rules! result_exports {
    ($(
        fn $name:ident($($arg:ident : $ty:ty),* $(,)?) -> $ret:ty;
    )+) => {
        $(
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn $name($($arg: $ty),*) -> $ret {
                wasm_impl::$name($($arg),*)
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn $name($($arg: $ty),*) -> $ret {
                let _ = ($(&$arg),*);
                Err(unsupported())
            }
        )+
    };
}

macro_rules! async_result_exports {
    ($(
        async fn $name:ident($($arg:ident : $ty:ty),* $(,)?) -> $ret:ty;
    )+) => {
        $(
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub async fn $name($($arg: $ty),*) -> $ret {
                wasm_impl::$name($($arg),*).await
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub async fn $name($($arg: $ty),*) -> $ret {
                let _ = ($(&$arg),*);
                Err(unsupported())
            }
        )+
    };
}

async_result_exports! {
    async fn register_graph_image(
        key: String,
        src: String,
    ) -> Result<(), wasm_bindgen::JsValue>;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn has_graph_image(key: &str) -> bool {
    wasm_impl::has_graph_image(key)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn has_graph_image(_key: &str) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn unregister_graph_image(key: &str) -> bool {
    wasm_impl::unregister_graph_image(key)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn unregister_graph_image(_key: &str) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn clear_graph_images() {
    wasm_impl::clear_graph_images()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_graph_images() {}

result_exports! {
    fn create_scatter_session(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<u32, wasm_bindgen::JsValue>;
    fn render_scatter_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_scatter_session(
        handle: u32,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn zoom_scatter_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_scatter_point_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn set_scatter_selection(
        handle: u32,
        index: Option<u32>,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn destroy_scatter_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn render_scatter(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_scatter(
        spec: wasm_bindgen::JsValue,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn pick_scatter_point(
        spec: wasm_bindgen::JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;

    fn render_tree(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn create_tree_session(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<u32, wasm_bindgen::JsValue>;
    fn render_tree_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn render_tree_session_transition(
        handle: u32,
        progress: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_tree_session(
        handle: u32,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn zoom_tree_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_tree_node_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn set_tree_selection(
        handle: u32,
        node_id: Option<String>,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn toggle_tree_node_collapsed_session(
        handle: u32,
        node_id: String,
    ) -> Result<bool, wasm_bindgen::JsValue>;
    fn set_tree_node_collapsed_session(
        handle: u32,
        node_id: String,
        collapsed: bool,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn destroy_tree_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_tree(
        spec: wasm_bindgen::JsValue,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn pick_tree_node(
        spec: wasm_bindgen::JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;

    fn create_line_session(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<u32, wasm_bindgen::JsValue>;
    fn render_line_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_line_session(
        handle: u32,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn zoom_line_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_line_point_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn set_line_selection(
        handle: u32,
        series_index: Option<u32>,
        point_index: Option<u32>,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn destroy_line_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn render_line(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_line(
        spec: wasm_bindgen::JsValue,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn pick_line_point(
        spec: wasm_bindgen::JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;

    fn create_bar_session(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<u32, wasm_bindgen::JsValue>;
    fn render_bar_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn zoom_bar_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_bar_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn set_bar_selection(
        handle: u32,
        series_index: Option<u32>,
        category_index: Option<u32>,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn destroy_bar_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn render_bar(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_bar(
        spec: wasm_bindgen::JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;

    fn create_heatmap_session(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<u32, wasm_bindgen::JsValue>;
    fn render_heatmap_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn zoom_heatmap_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_heatmap_cell_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn set_heatmap_selection(
        handle: u32,
        row: Option<u32>,
        col: Option<u32>,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn destroy_heatmap_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn render_heatmap(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_heatmap_cell(
        spec: wasm_bindgen::JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;

    fn create_network_session(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<u32, wasm_bindgen::JsValue>;
    fn update_network_session(
        handle: u32,
        spec: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn render_network_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn render_network_transition_session(
        handle: u32,
        progress: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_network_session(
        handle: u32,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn zoom_network_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pick_network_node_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn pick_network_hit_session(
        handle: u32,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn set_network_selection(
        handle: u32,
        node_id: Option<String>,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn set_network_tracking_path_session(
        handle: u32,
        node_ids: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn set_network_tracking_progress_session(
        handle: u32,
        progress: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn set_network_tracking_breath_phase_session(
        handle: u32,
        phase: f64,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn clear_network_tracking_path_session(
        handle: u32,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn get_network_view_session(
        handle: u32,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn set_network_view_session(
        handle: u32,
        view: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn compute_network_focus_view_session(
        handle: u32,
        node_id: String,
        options: Option<wasm_bindgen::JsValue>,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn has_network_transition_session(
        handle: u32,
    ) -> Result<bool, wasm_bindgen::JsValue>;
    fn clear_network_transition_session(
        handle: u32,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn destroy_network_session(handle: u32) -> Result<(), wasm_bindgen::JsValue>;
    fn render_network(
        canvas_id: &str,
        spec: wasm_bindgen::JsValue,
    ) -> Result<(), wasm_bindgen::JsValue>;
    fn pan_network(
        spec: wasm_bindgen::JsValue,
        delta_x: f64,
        delta_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn pick_network_node(
        spec: wasm_bindgen::JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    fn pick_network_hit(
        spec: wasm_bindgen::JsValue,
        canvas_x: f64,
        canvas_y: f64,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
}
