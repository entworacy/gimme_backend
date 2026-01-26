use super::{OAuthProvider, OAuthUserInfo};
use crate::shared::error::{AppError, AppResult};
use async_trait::async_trait;
use reqwest::Client;

use serde::Deserialize;

pub struct KakaoProvider {
    client_id: String,
    redirect_uri: String,
    client: Client,
}

impl KakaoProvider {
    pub fn new(client_id: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            redirect_uri,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct KakaoTokenResponse {
    access_token: String,
}

#[derive(Deserialize, Debug)]
struct KakaoUserAccount {
    email: Option<String>,
    age_range: Option<String>,
    birthyear: String,
    phone_number: Option<String>,
    profile: Option<KakaoUserProfile>,
}

#[derive(Deserialize, Debug)]
struct KakaoUserProfile {
    nickname: Option<String>,
}

#[derive(Deserialize, Debug)]
struct KakaoUserResponse {
    id: i64,
    connected_at: Option<String>,
    kakao_account: Option<KakaoUserAccount>,
}

#[async_trait]
impl OAuthProvider for KakaoProvider {
    fn get_authorization_url(&self) -> String {
        format!(
            "https://kauth.kakao.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code",
            self.client_id, self.redirect_uri
        )
    }

    async fn get_user_info(&self, code: &str) -> AppResult<OAuthUserInfo> {
        // 1. Get Access Token
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("code", code),
        ];

        let token_res = self
            .client
            .post("https://kauth.kakao.com/oauth/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Kakao token request failed: {}", e))
            })?
            .json::<KakaoTokenResponse>()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Kakao token parse failed: {}", e))
            })?;

        // 2. Get User Info
        let user_res = self
            .client
            .get("https://kapi.kakao.com/v2/user/me")
            .bearer_auth(token_res.access_token)
            .send()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Kakao user info request failed: {}", e))
            })?
            .json::<KakaoUserResponse>()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Kakao user info parse failed: {}", e))
            })?;

        let email = user_res
            .kakao_account
            .as_ref()
            .and_then(|a| a.email.clone());
        let name = user_res
            .kakao_account
            .as_ref()
            .and_then(|a| a.profile.as_ref())
            .and_then(|p| p.nickname.clone());
        let age_range = user_res
            .kakao_account
            .as_ref()
            .and_then(|a| a.age_range.clone());
        let birthyear = user_res.kakao_account.as_ref().unwrap().birthyear.clone();
        let phone_number = user_res
            .kakao_account
            .as_ref()
            .and_then(|a| a.phone_number.clone());

        Ok(OAuthUserInfo {
            provider_id: user_res.id.to_string(),
            email,
            age_range,
            birthyear,
            phone_number,
            name,
            connected_at: user_res.connected_at,
        })
    }
}
