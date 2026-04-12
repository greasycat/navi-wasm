use std::{net::SocketAddr, path::PathBuf};

use axum::{
    Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use tower_http::services::ServeDir;
use tracing::info;

/// Adds `Content-Type: application/wasm` for `.wasm` files, working around
/// systems whose MIME database lacks the type.
async fn fix_wasm_mime(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    if path.ends_with(".wasm") {
        response
            .headers_mut()
            .insert("content-type", "application/wasm".parse().unwrap());
    }
    response
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=info,tower_http=info".into()),
        )
        .init();

    // Serve from the workspace root so that /demo/ and /pkg/ both resolve.
    let root = workspace_root();

    let app = Router::new()
        .nest_service("/", ServeDir::new(&root))
        .layer(middleware::from_fn(fix_wasm_mime));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Serving {} on http://{}", root.display(), addr);
    info!("Demo:  http://{}/demo/", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Walk up from the binary location until we find a directory that contains
/// both `demo/` and `pkg/`, falling back to the current working directory.
fn workspace_root() -> PathBuf {
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    for _ in 0..10 {
        if dir.join("demo").is_dir() && dir.join("pkg").is_dir() {
            return dir;
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break,
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
