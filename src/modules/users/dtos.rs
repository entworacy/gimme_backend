use super::entities::social::SocialProvider;

pub struct SocialLoginDto {
    pub provider: SocialProvider,
    pub provider_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub connected_at: Option<String>,
}
