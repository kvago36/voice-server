use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{
    Error,
    dev::{Service, Transform, forward_ready},
    web,
};
use std::rc::Rc;
use std::{
    future::{Future, Ready, ready},
    pin::Pin,
};

use crate::state::State;

pub struct YaCloud;

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for YaCloud
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = YaCloudMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(YaCloudMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct YaCloudMiddleware<S> {
    /// The next service to call
    service: Rc<S>,
}

// This future doesn't have the requirement of being `Send`.
// See: futures_util::future::LocalBoxFuture
type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

// `S`: type of the wrapped service
// `B`: type of the body - try to be generic over the body where possible
impl<S, B> Service<ServiceRequest> for YaCloudMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    // This service is ready when its next service is ready
    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let mut app_data = req.app_data::<web::Data<State>>().unwrap();

            app_data.update_token().await.unwrap();

            let res = service.call(req).await?;

            Ok(res)
        })
    }
}
