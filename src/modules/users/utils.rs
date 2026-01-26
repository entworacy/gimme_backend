use sha2::{Digest, Sha512};

pub fn generate_user_uuid(provider_id: &str, connected_at: &str) -> String {
    let now_unix = chrono::Utc::now().timestamp();
    let raw = format!("{}|{}|JAY|PROP|{}", provider_id, now_unix, connected_at);
    let mut hasher = Sha512::new();
    hasher.update(raw);
    let result = hasher.finalize();
    format!("{:x}", result)
}
