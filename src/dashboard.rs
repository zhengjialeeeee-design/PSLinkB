//! Local read-only dashboard HTTP server.
//! Control endpoints are intentionally not exposed until state-machine commands
//! and LAN authentication are implemented.
use axum::{extract::State, http::{header, StatusCode}, response::{Html, IntoResponse}, routing::get, Json, Router};
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Clone)]
struct DashboardState {
    room_id: u64,
    douyin_configured: bool,
}

#[derive(Serialize)]
struct StatusResponse {
    service: &'static str,
    version: &'static str,
    integration: &'static str,
    room_id: u64,
    ps5: ComponentStatus,
    platforms: PlatformStatus,
    note: &'static str,
}
#[derive(Serialize)]
struct ComponentStatus { state: &'static str }
#[derive(Serialize)]
struct PlatformStatus {
    bilibili: ComponentStatus,
    douyin: ComponentStatus,
}

async fn index() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], Html(include_str!("../dashboard/index.html")))
}
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"ok": true, "service": "pslinkb-dashboard"}))
}
async fn status(State(state): State<Arc<DashboardState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        service: "PSLinkB",
        version: env!("CARGO_PKG_VERSION"),
        integration: "read-only-bootstrap",
        room_id: state.room_id,
        ps5: ComponentStatus { state: "not-yet-instrumented" },
        platforms: PlatformStatus {
            bilibili: ComponentStatus { state: "managed-by-existing-core" },
            douyin: ComponentStatus { state: if state.douyin_configured { "configured-not-yet-integrated" } else { "not-configured" } },
        },
        note: "Live telemetry and control actions are not wired yet; no stream credentials are exposed.",
    })
}

/// Starts the dashboard on loopback only. LAN binding is deliberately deferred
/// until token authentication and CSRF/Origin protections are implemented.
pub async fn serve(config: crate::config::Config) -> Result<(), std::io::Error> {
    let state = Arc::new(DashboardState {
        room_id: config.live.room_id,
        douyin_configured: config.live.douyin_rtmp_url.as_deref().is_some_and(|v| !v.is_empty())
            && config.live.douyin_stream_key.as_deref().is_some_and(|v| !v.is_empty()),
    });
    let app = Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        .route("/api/status", get(status))
        .with_state(state);
    let addr: SocketAddr = "127.0.0.1:8787".parse().expect("valid dashboard bind address");
    let listener = TcpListener::bind(addr).await?;
    eprintln!("[Dashboard] Read-only UI: http://{}", addr);
    axum::serve(listener, app).await
}
