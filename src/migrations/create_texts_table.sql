CREATE TABLE IF NOT EXISTS texts (
                                     id SERIAL PRIMARY KEY,
                                     user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                     content TEXT NOT NULL,
                                     created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
)