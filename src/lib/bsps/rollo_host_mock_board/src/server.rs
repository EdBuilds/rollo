use std::borrow::BorrowMut;
use std::ops::Deref;
use std::sync::{Arc};
use futures::lock::Mutex;
use std::thread;

use tide::{Endpoint, Request, Response, Route};
use bal::server::*;
use async_std::task::block_on;
use http_types;
use async_trait::async_trait;
use futures::AsyncReadExt;
use http_types::StatusCode;
use crate::ServerContainer;
impl ServerResource for ServerContainer {
    fn create_server(&mut self, handlers: Vec<bal::server::Handler>) -> Result<(), Error> {
        let mut app = tide::new();
        for handler in handlers {
            let method: http_types::Method = match handler.method {
                bal::networking_types::Method::Get => {http_types::Method::Get}
                bal::networking_types::Method::Put => {http_types::Method::Put}
                bal::networking_types::Method::Post => {http_types::Method::Post}
                bal::networking_types::Method::Delete => {http_types::Method::Delete}
            };
            app.at(handler.uri.as_str()).method(method, EndpointWrapper(Arc::new(Mutex::new(handler.handler))));
        }
        thread::spawn(move  || {
            let listener = "127.0.0.1:80";
            println!("Server starting on: {}", listener);
            println!("Server crapped itself:{:#?}",block_on(app.listen(listener)));
        });
        Ok(())
    }
}

struct EndpointWrapper(Arc<Mutex<Box<dyn Fn(String) -> bal::networking_types::Response + Send>>>);

#[async_trait]
impl Endpoint<()> for EndpointWrapper
{

    async fn call(&self, mut req: Request<()>) -> tide::Result {
        let req_body_str = req.take_body().into_string().await?;
        let mut result = self.0.lock().await.deref()(req_body_str);
        let http_status_code = match result.status {
            bal::networking_types::Status::Ok => {http_types::StatusCode::Ok}
            bal::networking_types::Status::InternalServerError => {http_types::StatusCode::InternalServerError}
            bal::networking_types::Status::BadRequest => {http_types::StatusCode::BadRequest}
        };
        let mut resp = Response::new(http_status_code);
        resp.set_body(result.body);
        Ok(resp)
    }
}
