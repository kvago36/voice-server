use actix::prelude::*;
use actix_session::{Session, SessionExt};
use actix_web::guard::{Guard, GuardContext};
use actix_web::{
    HttpRequest, HttpResponse,
    error::{Error, ErrorBadRequest},
    get,
    http::StatusCode,
    post, rt, web,
};
use futures_util::{SinkExt, StreamExt as _};

use crate::models::User;
use crate::session::UserSession;
use crate::state::State;

#[post("/{id}")]
async fn save_user_data(
    info: web::Path<(String, usize)>,
    session: Session,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::new(StatusCode::OK))
}

#[get("/{id}")]
async fn get_user_data(
    info: web::Path<(String, usize)>,
    session: Session,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::new(StatusCode::OK))
}

#[get("/")]
async fn get_users(data: web::Data<State>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let mut rows = data.pool.query("SELECT * FROM users", ()).await.unwrap();
    let mut users = Vec::new();

    while let Some(row) = rows.next().await.unwrap() {
        let id = row.get(0).unwrap();
        let name: String = row.get(1).unwrap();
        users.push(User::new(id, name, row.get(2).unwrap()));
    }

    Ok(HttpResponse::Ok().json(users))
}

struct MyGuard;

impl Guard for MyGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        ctx.get_session()
            .get::<UserSession>("session_id")
            .unwrap()
            .is_some()
    }
}

pub fn users_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .guard(MyGuard)
            .service(get_users)
            .service(get_user_data)
            .service(save_user_data),
    );
}
