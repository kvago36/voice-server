use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    user_id: i64,
    username: String,
    texts_count: i64,
    created_at: String,
}

impl User {
    pub fn new(user_id: i64, username: String, texts_count: i64, created_at: String) -> Self {
        User {
            user_id,
            username,
            texts_count,
            created_at,
        }
    }
}
