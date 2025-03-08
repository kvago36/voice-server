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
async fn save_user_data(
    info: web::Json<UpdateUser>,
    path: web::Path<i64>,
    data: web::Data<State>,
    session: Session,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let pool = &data.pool;
    let user_id = path.into_inner();
    let text = info.text.clone();
    let query = sqlx::query("INSERT INTO texts (user_id, content) VALUES ($1, $2)")
        .bind(user_id)
        .bind(text);

    // TODO: check is user exist and no unwrap
    let result = pool.execute(query).await.unwrap();

    if result.rows_affected() == 1 {
        Ok(HttpResponse::new(StatusCode::OK))
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
    let rows = sqlx::query("SELECT * FROM texts WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap();

    for row in &rows {
        let content: String = row.get(2);
        let created_at: DateTime<Utc> = row.get(3);

        texts.push(UserText { content, created_at: created_at.to_string() });
    }

    let json = json!({ "text": texts });

    Ok(HttpResponse::Ok().json(json))
}

#[post("/")]
async fn create_user(
    info: web::Json<CreateUser>,
    data: web::Data<State>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let pool = &data.pool;
    let query = sqlx::query("INSERT INTO users (username) VALUES ($1)").bind(&info.username);
    let result = pool.execute(query).await.unwrap();

    if result.rows_affected() > 0 {
        let json = json!({ "username": &info.username });

        session.insert("session_id", &json)?;
        Ok(HttpResponse::Ok().json(json))
    } else {
        Ok(HttpResponse::new(StatusCode::NOT_FOUND))
    }
}

#[get("/")]
async fn get_users(data: web::Data<State>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let pool = &data.pool;
    let query = sqlx::query("SELECT * FROM users ORDER BY created_at DESC");
    let mut result = pool.execute(query).await.unwrap();
    // let mut users = Vec::new();

    Ok(HttpResponse::new(StatusCode::OK))
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
            .service(save_user_data),
    );
}
