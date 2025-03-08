-- migrations.sql

-- Таблица пользователей
CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                        user_name TEXT NOT NULL,
                       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        texts_count NUMERIC DEFAULT
);

-- Таблица постов (текстов)
CREATE TABLE texts (
                       id SERIAL PRIMARY KEY,
                       user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                       content TEXT NOT NULL,
                       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);