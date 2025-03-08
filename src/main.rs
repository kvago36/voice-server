use actix::prelude::*;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::body::MessageBody;
use actix_web::{App, HttpServer, cookie::Key, web};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::Executor;
use sqlx::migrate::Migrate;
use sqlx_postgres::PgPool;
use std::env;
use tonic::IntoRequest;
use tonic::service::Interceptor;
use tonic::transport::Body;

mod handlers;
mod middlewares;
mod models;
mod session;
mod state;
mod ws;

use crate::state::TokenInfo;
use handlers::{auth, users};
use middlewares::stt::YaCloud;
use state::State;
use ws::handle_message;

pub mod api {
    tonic::include_proto!("speechkit.stt.v3");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let ya_cloud_url = env::var("YA_CLOUD_URL").expect("Cant find .env in YA_CLOUD_URL");

    let oauth_token = env::var("OAUTH_TOKEN").expect("Cant find .env in OAUTH_TOKEN");
    let folder_id = env::var("FOLDER_ID").expect("Cant find .env in FOLDER_ID");

    let db_url = env::var("DB_URL").expect("Cant find DB_URL in .env");

    let pool = PgPool::connect(&db_url).await.unwrap();

    let users_query = sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(50) NOT NULL UNIQUE,
            texts_count INT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP)",
    );

    let texts_query = sqlx::query(
        "CREATE TABLE IF NOT EXISTS texts (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                content TEXT NOT NULL,
                created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
            );",
    );

    pool.execute(users_query).await.unwrap();
    pool.execute(texts_query).await.unwrap();

    let cookie_key = Key::generate();

    let token_info = TokenInfo::from_file("./token.json");
    let mut state = State::new(&ya_cloud_url, pool, oauth_token, folder_id, token_info);

    state.update_token().await.unwrap();

    let app_state = web::Data::new(state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/ws", web::get().to(handle_message).wrap(YaCloud))
            .service(
                web::scope("/api")
                    .configure(auth::auth::auth_config)
                    .configure(users::users::users_config),
            )
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                cookie_key.clone(),
            ))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
