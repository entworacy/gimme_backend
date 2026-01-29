use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_connect_timeout: u64,
    pub database_idle_timeout: u64,
    pub server_host: String,
    pub server_port: u16,
    pub rust_log: String,
    pub app_env: String,
    pub kakao_client_id: String,
    pub kakao_redirect_uri: String,
    pub gmail_user: String,
    pub gmail_app_password: String,
    pub redis_url: String,
}

impl Config {
    pub fn init() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid number");
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

        // Kakao Config (Optional in dev if not used, but good to enforce if feature is active)
        let kakao_client_id = env::var("KAKAO_CLIENT_ID").unwrap_or_else(|_| "".to_string());
        let kakao_redirect_uri = env::var("KAKAO_REDIRECT_URI").unwrap_or_else(|_| "".to_string());

        // Gmail Config
        let gmail_user = env::var("GMAIL_USER").unwrap_or_else(|_| "".to_string());
        let gmail_app_password = env::var("GMAIL_APP_PASSWORD").unwrap_or_else(|_| "".to_string());

        // Redis Config
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());

        Self {
            database_url,
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse::<u32>()
                .expect("DATABASE_MAX_CONNECTIONS must be a valid number"),
            database_min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string()) // Minimum connections to keep alive
                .parse::<u32>()
                .expect("DATABASE_MIN_CONNECTIONS must be a valid number"),
            database_connect_timeout: env::var("DATABASE_CONNECT_TIMEOUT")
                .unwrap_or_else(|_| "8".to_string())
                .parse::<u64>()
                .expect("DATABASE_CONNECT_TIMEOUT must be a valid number"),
            database_idle_timeout: env::var("DATABASE_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "8".to_string())
                .parse::<u64>()
                .expect("DATABASE_IDLE_TIMEOUT must be a valid number"),
            server_host,
            server_port,
            rust_log,
            app_env,
            kakao_client_id,
            kakao_redirect_uri,
            gmail_user,
            gmail_app_password,
            redis_url,
        }
    }
}
