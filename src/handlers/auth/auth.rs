use actix_session::{Session, SessionExt};
use actix_web::error::ErrorBadRequest;
use actix_web::http::StatusCode;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use serde_json::json;

use crate::session::UserSession;

async fn auth(session: Session, request: HttpRequest) -> Result<HttpResponse, Error> {
    if let Some(_) = session.get::<UserSession>("session_id")? {
        Err(ErrorBadRequest("You already logged in!"))
    } else {
        request
            .get_session()
            .insert("session_id", json!({ "user_id": "12" }))?;
        Ok(HttpResponse::new(StatusCode::OK))
    }
}

async fn logout(session: Session, request: HttpRequest) -> Result<HttpResponse, Error> {
    if let Some(_) = session.remove("session_id") {
        Ok(HttpResponse::new(StatusCode::OK))
    } else {
        Err(ErrorBadRequest("You are not logged in"))
    }
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(auth))
            .route("/logout", web::post().to(logout)),
    );
}
