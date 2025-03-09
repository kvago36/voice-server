use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserSession {
    user_id: String,
    user_name: String,
}
