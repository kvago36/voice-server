use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    id: i64,
    user_name: String,
    texts_count: i64,
    created_at: String,
}

impl User {
    pub fn new(id: i64, user_name: String, texts_count: i64, created_at: String) -> Self {
        User {
            id,
            user_name,
            texts_count,
            created_at,
        }
    }
}
