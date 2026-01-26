use axum::{Router, routing::get};
use gimme_backend::{
    modules::{
        self,
        auth::{providers::kakao::KakaoProvider, registry::OAuthProviderRegistry},
        users::entities::social::SocialProvider,
    },
    shared::{config::Config, db, state::AppState},
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
    // Configure Repository based on Env
    let user_repo: Arc<dyn modules::users::repository::UserRepository> = if config.app_env == "dev"
    {
        tracing::warn!("Using InMemory Repository for Dev Env");
        Arc::new(modules::users::repository::InMemoryUserRepository::new())
    } else {
        let db_conn = db::connect(&config)
            .await
            .expect("Failed to connect to database");
        tracing::info!("Connected to database");
        Arc::new(modules::users::repository::PostgresUserRepository::new(
            Arc::new(db_conn),
        ))
    };

    tracing::info!("Current App Env: {}", config.app_env);

    let config_arc = Arc::new(config.clone());

    let auth_registry = OAuthProviderRegistry::new().register(
        SocialProvider::Kakao,
        KakaoProvider::new(
            config.kakao_client_id.clone(),
            config.kakao_redirect_uri.clone(),
        ),
    );

    let app_state = AppState {
        config: config_arc,
        auth_registry: auth_registry.clone(), // Registry might need to be Arc-ed or Clone is cheap? It's a HashMap inside?
        user_repo,
    };

    // Initialize router
    // Aggregate routes from modules
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/users", modules::users::router::router(app_state.clone()))
        .nest("/auth", modules::auth::router::router(app_state));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
