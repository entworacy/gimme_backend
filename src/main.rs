use axum::{Router, routing::get};
use gimme_backend::{
    modules::{
        self,
        auth::{providers::kakao::KakaoProvider, registry::OAuthProviderRegistry},
        users::entities::social::SocialProvider,
    },
    shared::{config::Config, db},
};
use std::net::SocketAddr;
use std::sync::Arc;
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

    // Connect to database
    let db_conn = db::connect(&config)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");
    tracing::info!("Current App Env: {}", config.app_env);

    let db = Arc::new(db_conn);
    let config_arc = Arc::new(config.clone()); // Assuming Config derives Clone, checking... yes it does.

    let auth_registry = OAuthProviderRegistry::new().register(
        SocialProvider::Kakao,
        KakaoProvider::new(
            config.kakao_client_id.clone(),
            config.kakao_redirect_uri.clone(),
        ),
    );

    // Initialize router
    // Aggregate routes from modules
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/users", modules::users::router::router(db.clone()))
        .nest(
            "/auth",
            modules::auth::router::router(db.clone(), config_arc.clone(), auth_registry),
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
