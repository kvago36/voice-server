use crate::models::User;
use crate::session::UserSession;
use crate::state::State;
use actix::fut::result;
use actix::prelude::*;
use actix_session::{Session, SessionExt};
use actix_web::guard::{Guard, GuardContext};
use actix_web::http::header::q;
use actix_web::{
    HttpRequest, HttpResponse,
    error::{Error, ErrorBadRequest},
    get,
    http::StatusCode,
    post, web,
};
use chrono::{DateTime, NaiveDate, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Encode, Executor, Row, query};
use tonic::codegen::tokio_stream::StreamExt;
use tonic::IntoRequest;

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}
#[derive(Deserialize)]
struct UpdateUser {
    text: String,
}

#[derive(Serialize)]
struct UserText {
    content: String,
    created_at: String,
}

#[post("/{id}")]
async fn save_user_text(
    info: web::Json<UpdateUser>,
    path: web::Path<i32>,
    data: web::Data<State>,
    session: Session,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let pool = &data.pool;
    let user_id = path.into_inner();
    let text = info.text.clone();
    let query = sqlx::query(include_str!("../../../src/sql/save_user_text.sql"))
        .bind(user_id)
        .bind(text);

    // TODO: check is user exist and no unwrap
    let result = pool.execute(query).await.unwrap();

    if result.rows_affected() == 1 {
        let rows = sqlx::query(include_str!("../../../src/sql/find_user_by_id.sql")).bind(user_id).fetch_all(pool).await.unwrap();

        let texts: Vec<UserText> = rows.iter().map(|row| {
            let content = row.get("content");
            let created_at: DateTime<Utc> = row.get("created_at");

            UserText {
                content,
                created_at: created_at.to_string(),
            }
        }).collect();

        let json = json!({ "texts": texts });

        Ok(HttpResponse::Ok().json(json))
    } else {
        Err(ErrorBadRequest("Cannot update user"))
    }
}
#[get("/{id}")]
async fn get_user_data(
    path: web::Path<i32>,
    data: web::Data<State>,
    session: Session,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user_id = path.into_inner();
    let pool = &data.pool;
    let mut texts = Vec::new();

    // TODO: check is user exist and no unwrap
    let rows = sqlx::query(include_str!("../../../src/sql/find_user_by_id.sql"))
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap();

    for row in &rows {
        let content: String = row.get(2);
        let created_at: DateTime<Utc> = row.get(3);

        texts.push(UserText { content, created_at: created_at.to_string() });
    }

    let json = json!({ "texts": texts });

    Ok(HttpResponse::Ok().json(json))
}

#[post("/")]
async fn create_user(
    info: web::Json<CreateUser>,
    data: web::Data<State>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let pool = &data.pool;

    let user_row = sqlx::query(include_str!("../../../src/sql/find_user_by_name.sql")).bind(&info.username).fetch_one(pool).await;

    if let Ok(user) = user_row {
        let user_id: i32 = user.get("id");
        let user_json = json!({ "user_id": user_id, "username": info.username });

        session.insert("session_id", &user_json)?;

        return Ok(HttpResponse::Ok().json(user_json))
    };

    let user_query = sqlx::query(include_str!("../../../src/sql/create_user.sql")).bind(&info.username).fetch_one(pool).await;

    user_query.map_or_else(|e| {
        Ok(HttpResponse::new(StatusCode::NOT_FOUND))
    }, |user| {
        let user_id: i32 = user.get("id");
        let user_json = json!({ "user_id": user_id, "username": info.username });

        session.insert("session_id", &user_json)?;

        Ok(HttpResponse::Ok().json(user_json))
    })
}

#[get("/")]
async fn get_users(data: web::Data<State>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let pool = &data.pool;
    let mut users = Vec::new();
    let mut users_stream = sqlx::query(include_str!("../../../src/sql/get_users.sql")).fetch(pool);

    while let Some(row) = users_stream.next().await {
        if let Ok(user) = row {
            let user_id: i32 = user.get("user_id");
            let user_name: String = user.get("username");
            let texts_count: i64 = user.get("texts_count");
            let created_at: DateTime<Utc> = user.get("created_at");

            users.push(User::new(user_id as i64, user_name, texts_count, created_at.to_string()));
        }
    }

    let users_json = json!({ "users": users });

    Ok(HttpResponse::Ok().json(users_json))
}

// struct MyGuard;
//
// impl Guard for MyGuard {
//     fn check(&self, ctx: &GuardContext<'_>) -> bool {
//         // if
//         println!("{:?}", ctx.head().uri.to_string());
//
//         // println!("{:#?}", data.into_request().);
//
//         ctx.get_session()
//             .get::<UserSession>("session_id")
//             .unwrap()
//             .is_some()
//     }
// }

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(create_user)
            .service(get_users)
            .service(get_user_data)
            .service(save_user_text),
    );
}
