use crate::shared::error::AppResult;
use async_trait::async_trait;

pub mod kakao;

#[derive(Debug)]
pub struct OAuthUserInfo {
    pub provider_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn get_authorization_url(&self) -> String;
    async fn get_user_info(&self, code: &str) -> AppResult<OAuthUserInfo>;
}
