use crate::shared::config::Config;
use crate::shared::error::{AppError, AppResult};
use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send_verification_code(&self, to: &str, code: &str) -> AppResult<()>;
}

pub struct GmailProvider {
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: String,
    app_env: String,
}

impl GmailProvider {
    pub fn new(config: &Config) -> Self {
        let app_env = config.app_env.clone();

        if app_env == "dev" || app_env == "test" {
            // In dev/test, we don't need actual mailer
            return Self {
                mailer: None,
                from: "dev@gimme.com".to_string(),
                app_env,
            };
        }

        let creds = Credentials::new(config.gmail_user.clone(), config.gmail_app_password.clone());

        // Open connection to Gmail
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .expect("Failed to build mailer") // Should propagate error properly in real app, but for now panic on init is "okay" or we can return Result
            .credentials(creds)
            .build();

        Self {
            mailer: Some(mailer),
            from: config.gmail_user.clone(),
            app_env,
        }
    }
}

#[async_trait]
impl EmailProvider for GmailProvider {
    async fn send_verification_code(&self, to: &str, code: &str) -> AppResult<()> {
        if self.app_env == "dev" || self.app_env == "test" {
            println!("--------------------------------------------------");
            println!("[DEV] Sending Verification Code to: {}", to);
            println!("[DEV] Code: {}", code);
            println!("--------------------------------------------------");
            return Ok(());
        }

        let email = Message::builder()
            .from(self.from.parse().map_err(|e| {
                AppError::InternalServerError(format!("Invalid from address: {}", e))
            })?)
            .to(to
                .parse()
                .map_err(|e| AppError::BadRequest(format!("Invalid to address: {}", e)))?)
            .subject("Gimme Verification Code")
            .header(ContentType::TEXT_PLAIN)
            .body(format!("Your verification code is: {}", code))
            .map_err(|e| AppError::InternalServerError(format!("Failed to build email: {}", e)))?;

        if let Some(mailer) = &self.mailer {
            mailer.send(email).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to send email: {}", e))
            })?;
        } else {
            return Err(AppError::InternalServerError(
                "Mailer not initialized in non-dev env".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gmail_provider_dev_mode() {
        let config = Config {
            database_url: "".to_string(),
            database_max_connections: 100,
            database_min_connections: 5,
            database_connect_timeout: 8,
            database_idle_timeout: 8,
            server_host: "localhost".to_string(),
            server_port: 3000,
            rust_log: "info".to_string(),
            app_env: "dev".to_string(),
            kakao_client_id: "".to_string(),
            kakao_redirect_uri: "".to_string(),
            gmail_user: "".to_string(),
            gmail_app_password: "".to_string(),
            redis_url: "".to_string(),
        };

        let provider = GmailProvider::new(&config);
        assert!(provider.mailer.is_none());
        assert_eq!(provider.app_env, "dev");

        let result = provider
            .send_verification_code("test@example.com", "123456")
            .await;
        assert!(result.is_ok());
    }
}
