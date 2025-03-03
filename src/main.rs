use actix::prelude::*;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpServer, cookie::Key, web};
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt as _};
use libsql::Builder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tonic::IntoRequest;
use tonic::service::Interceptor;
use tonic::transport::Body;

mod handlers;
mod models;
mod session;
mod state;
mod ws;

use handlers::{auth, users};
use state::State;
use ws::handle_message;

pub mod api {
    tonic::include_proto!("speechkit.stt.v3");
}

#[derive(Deserialize)]
struct TokenResponse {
    #[serde(rename = "iamToken")]
    iam_token: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let oauth_token = env::var("OAUTH_TOKEN").unwrap().to_string();
    let folder_id = env::var("FOLDER_ID").unwrap().to_string();

    let mut token = Some("t1.9euelZrGkpyUycmcxp2Ni5bOzJqMke3rnpWaj5rJiZOSlZqcnIvNy5iUlcbl8_cKFwBC-e9kIHhM_d3z90pFfUH572QgeEz9zef1656VmpyUz5uSxpmRmZzOxo-Qz42X7_zF656VmpyUz5uSxpmRmZzOxo-Qz42X.XeXVbwExKJiJE7UTovyL-1gYyq6Vu22onaXSImXJ1xEv36_VKmwjr0bJhGN-NitWu5rwQzuWgMJgEd6Rnv4PCg".to_string());
    // let url = "https://iam.api.cloud.yandex.net/iam/v1/tokens";
    // let reqwest_client = reqwest::Client::new();
    // let res = reqwest_client.post(url)
    //     .body(json!({ "yandexPassportOauthToken": oauth_token }).to_string())
    //     .send()
    //     .await
    //     .unwrap();
    //
    // if res.status() == 200 {
    //     let data = res.json::<TokenResponse>().await.unwrap();
    //     token = Some(data.iam_token);
    // };
    //
    // println!("{:?}", token);

    let db_url = "libsql://the-krusty-krab-branch-001-kvago.turso.io";
    let db_token = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhIjoicnciLCJleHAiOjE3NDEzNjA5OTEsImlhdCI6MTc0MDg0MjU5MSwiaWQiOiI0NWQyNzFlYy1hY2IzLTQ0OTktODQ3My1jYjRkMTA5MDI1NTgifQ.0I7XlWnn6GYLwT4bhkFD1Txo1JEUm94YIO2KxDbq4gjGXtZp9NX6aPX3tZ-4FrDw3bQTtECjBSHvT77yLieFCA";

    let db = Builder::new_remote(db_url.to_string(), db_token.to_string())
        .build()
        .await
        .unwrap();
    let conn = db.connect().unwrap();

    let cookie_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State::new(
                conn.clone(),
                token.as_ref().unwrap().clone(),
                folder_id.to_string(),
            )))
            .service(
                web::scope("/api")
                    .configure(auth::auth::auth_config)
                    .configure(users::users::users_config),
            )
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                cookie_key.clone(),
            ))
            .route("/ws", web::get().to(handle_message))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
