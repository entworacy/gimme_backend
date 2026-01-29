use crate::modules::users::dtos::SocialLoginDto;
use crate::modules::users::entities::{
    social::{self},
    user,
};
use crate::modules::users::repository::UserRepository;
use crate::shared::error::{AppError, AppResult};
use sea_orm::ActiveValue::Set;

pub struct UserService;

impl UserService {
    pub async fn handle_social_login(
        repo: &dyn UserRepository,
        login_dto: SocialLoginDto,
    ) -> AppResult<user::Model> {
        // 1. Check if Social Account exists
        let social_account = repo
            .find_social(login_dto.provider.clone(), &login_dto.provider_id)
            .await?;

        if let Some(social) = social_account {
            let user =
                repo.find_by_id(social.user_id)
                    .await?
                    .ok_or(AppError::InternalServerError(
                        "User not found for social account".to_string(),
                    ))?;

            // Logic a: Check status
            match user.account_status {
                crate::modules::users::entities::enums::AccountStatus::Active => return Ok(user),
                crate::modules::users::entities::enums::AccountStatus::Pending => {
                    // TODO: Re-trigger Email Verification Logic here
                    // For now, just return user to allow login/proceed (or maybe we shouldn't login?)
                    // The requirement says: "별도의 DB 삽입없이 Email 인증만 새로 생성하고"
                    // implies we might just want to send email again.
                    // But if this is "Login", we probably return the user so they can theoretically get a token?
                    // OR if the flow is Strict Registration, maybe we return OK but frontend handles it?
                    // Assuming we return User for now.
                    return Ok(user);
                }
                _ => return Ok(user), // Inactive/Suspended?
            }
        }

        // 2. Create new User
        // UUID Logic: SHA512(provider_id|unix|JAY|PROP|connected_at)
        let connected_at = login_dto
            .connected_at
            .clone()
            .ok_or(AppError::BadRequest("Missing connected_at".to_string()))?;
        let new_uuid =
            crate::modules::users::utils::generate_user_uuid(&login_dto.provider_id, &connected_at);

        let now = chrono::Utc::now().naive_utc();

        // Prepare User ActiveModel
        let username = login_dto.name.unwrap_or_else(|| "User".to_string());
        let email = login_dto.email.unwrap_or_else(|| "".to_string());

        if username == "User" {
            return Err(AppError::BadRequest(
                "Username or email is missing (maybe oauth provider's issue)".to_string(),
            ));
        }

        // Prepare User ActiveModel
        let new_user = user::ActiveModel {
            uuid: Set(new_uuid),
            username: Set(username),
            email: Set(email),
            country_code: Set("".to_string()),
            phone_number: Set(login_dto.phone_number.unwrap_or_else(|| "".to_string())),
            account_status: Set(crate::modules::users::entities::enums::AccountStatus::Pending),
            created_at: Set(now),
            updated_at: Set(now),
            last_login_at: Set(Some(now)),
            ..Default::default()
        };

        // Prepare Social ActiveModel
        let new_social = social::ActiveModel {
            provider: Set(login_dto.provider),
            provider_id: Set(login_dto.provider_id),
            created_at: Set(now),
            ..Default::default()
        };

        // Prepare Verification ActiveModel
        let new_verification = crate::modules::users::entities::verification::ActiveModel {
            email_verified: Set(false),
            phone_verified: Set(false),
            business_verified: Set(false),
            business_info: Set(Some("{}".to_string())), // Default JSON string
            verification_code: Set(None),
            ..Default::default()
        };

        // Delegate to Repo
        let created_user = repo
            .create_user_with_verification(new_user.clone(), Some(new_social), new_verification)
            .await?;

        // TODO: Send Verification Email here

        Ok(created_user)
    }
}
