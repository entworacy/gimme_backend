use crate::modules::auth::{
    providers::{
        email::{EmailProvider, GmailProvider},
        kakao::KakaoProvider,
    },
    registry::OAuthProviderRegistry,
};
use crate::modules::users::entities::social::SocialProvider;
use crate::shared::config::Config;
use std::sync::Arc;

pub fn init_auth_registry(config: &Config) -> OAuthProviderRegistry {
    OAuthProviderRegistry::new().register(
        SocialProvider::Kakao,
        KakaoProvider::new(
            config.kakao_client_id.clone(),
            config.kakao_redirect_uri.clone(),
        ),
    )
}

pub fn init_email_provider(config: &Config) -> Arc<dyn EmailProvider> {
    Arc::new(GmailProvider::new(config))
}
