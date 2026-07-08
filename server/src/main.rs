mod db;
mod handlers;
mod auth;

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<db::Database>,
    pub jwt_secret: String,
    pub broadcast_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database = db::Database::new("openiot.db").expect("Failed to init database");
    let (broadcast_tx, _) = broadcast::channel::<String>(128);
    let state = AppState {
        db: Arc::new(database),
        jwt_secret: uuid::Uuid::new_v4().to_string(),
        broadcast_tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/register", post(handlers::register))
        .route("/api/login", post(handlers::login))
        .route("/api/devices", get(handlers::list_devices).post(handlers::add_device))
        .route("/api/devices/:id", get(handlers::get_device).delete(handlers::delete_device))
        .route("/api/devices/:id/command", post(handlers::send_command))
        .route("/api/provision", post(handlers::provision_device))
        .route("/ws", get(handlers::ws_handler))
        .nest_service("/", ServeDir::new("server/static"))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server started on http://localhost:3000");
    tracing::info!("API: http://localhost:3000/api/*");
    axum::serve(listener, app).await.unwrap();
}
