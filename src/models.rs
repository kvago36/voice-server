use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    id: u64,
    name: String,
    texts_count: u64,
}

impl User {
    pub fn new(id: u64, name: String, texts_count: u64) -> Self {
        User {
            id,
            name,
            texts_count,
        }
    }
}
