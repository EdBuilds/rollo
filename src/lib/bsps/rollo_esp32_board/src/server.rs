use std::borrow::BorrowMut;
use std::io::Read;
use embedded_svc::httpd::{Body, Method, Request, Response};
use embedded_svc::httpd::registry::{MiddlewareRegistry, Registry};
use esp_idf_svc;
use esp_idf_svc::httpd::Configuration;
use bal::networking_types::Status;
use bal::server::{ServerResource, Error};


type WrappedServer = esp_idf_svc::httpd::Server;
pub struct EspWebServer{
    server: Option<WrappedServer>
}
impl EspWebServer {
    pub fn new()-> EspWebServer {
        EspWebServer{server: None}
    }
}
impl ServerResource for EspWebServer {
    fn create_server(&mut self, handlers: Vec<bal::server::Handler>) -> Result<(),Error>{
        let mut server_reg = esp_idf_svc::httpd::ServerRegistry::new();

        for handle in handlers{
            let server_reg_builder = server_reg.at(handle.uri);
            let what  = (move |mut rq: embedded_svc::httpd::Request|{
                let mut buf = String::new();
                rq.read_to_string(buf.borrow_mut()).unwrap();
                let handl_out: bal::networking_types::Response = (handle.handler)(buf);
                let status_code = match handl_out.status {
                    Status::Ok => {200}
                    Status::InternalServerError => {400}
                    Status::BadRequest => {500}
                };
                let mut resp = Response::new(status_code);
                Ok(resp.body(Body::Bytes(handl_out.body.into_bytes())))
            });
            server_reg = match handle.method{
                bal::networking_types::Method::Get => {server_reg_builder.get(what).map_err(|_|Error::Register)?}
                bal::networking_types::Method::Put => {server_reg_builder.put(what).map_err(|_|Error::Register)?}
                _ => {return Err(Error::Undefined)}
            }

        }
        self.server = Some(server_reg.start(&Configuration::default()).map_err(|_|Error::Start)?);
        Ok(())

    }
}
pub fn create_server(registry: embedded_svc::httpd::registry::MiddlewareRegistry) {
}