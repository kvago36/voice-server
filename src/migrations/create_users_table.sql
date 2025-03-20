CREATE TABLE IF NOT EXISTS users (
                                     id SERIAL PRIMARY KEY,
                                     username VARCHAR(50) NOT NULL UNIQUE,
                                     created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP)