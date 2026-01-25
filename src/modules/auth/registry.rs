use super::providers::OAuthProvider;
use crate::modules::users::entities::social::SocialProvider;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct OAuthProviderRegistry {
    providers: HashMap<String, Arc<dyn OAuthProvider>>,
}

impl OAuthProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register<P: OAuthProvider + 'static>(
        mut self,
        provider_type: SocialProvider,
        provider: P,
    ) -> Self {
        self.providers
            .insert(Self::key(provider_type), Arc::new(provider));
        self
    }

    pub fn get(&self, provider_type: SocialProvider) -> Option<Arc<dyn OAuthProvider>> {
        self.providers.get(&Self::key(provider_type)).cloned()
    }

    fn key(provider_type: SocialProvider) -> String {
        match provider_type {
            SocialProvider::Kakao => "KAKAO".to_string(),
            SocialProvider::Google => "GOOGLE".to_string(),
            SocialProvider::Apple => "APPLE".to_string(),
        }
    }
}
