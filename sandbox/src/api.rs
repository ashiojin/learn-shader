use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::mpsc::Sender;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    reload_tx: Sender<()>,
}

pub fn spawn_api_server(reload_tx: Sender<()>) {
    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("API_PORT must be a valid u16");

    let state = AppState { reload_tx };

    let app = Router::new()
        .route("/wgsl/:type", get(read_wgsl))
        .route("/wgsl/:type", post(write_wgsl))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .expect("Failed to bind API server");
        println!("API server listening on 0.0.0.0:{}", port);
        axum::serve(listener, app).await.expect("Failed to run API server");
    });
}

async fn read_wgsl(Path(shader_type): Path<String>) -> impl IntoResponse {
    if shader_type == "ExtendedMaterial" {
        (StatusCode::OK, crate::sample::extended_material::read_global_res())
    } else {
        (StatusCode::NOT_FOUND, "Unknown shader type".to_string())
    }
}

async fn write_wgsl(
    Path(shader_type): Path<String>,
    State(state): State<AppState>,
    body: String,
) -> impl IntoResponse {
    if shader_type == "ExtendedMaterial" {
        crate::sample::extended_material::write_global_res(&body);
        let _ = state.reload_tx.send(());
        (StatusCode::OK, "WGSL updated".to_string())
    } else {
        (StatusCode::NOT_FOUND, "Unknown shader type".to_string())
    }
}
