use axum::{Router, routing::get};
use gimme_backend::shared::handlers::{handler_404, handler_500};
use gimme_backend::{bootstrap, modules, shared::config::Config};
use std::net::SocketAddr;
use tower_http::catch_panic::CatchPanicLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize config
    let config = Config::init();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Current App Env: {}", config.app_env);

    // Bootstrap AppState
    let app_state = bootstrap::create_app_state(&config).await;

    // Initialize router
    // Aggregate routes from modules
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/users", modules::users::router::router(app_state.clone()))
        .nest("/auth", modules::auth::router::router(app_state))
        .layer(CatchPanicLayer::custom(handler_500))
        .fallback(handler_404);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
