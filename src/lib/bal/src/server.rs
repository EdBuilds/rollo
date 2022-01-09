use embedded_svc::httpd::registry::Registry;
use crate::networking_types::{Method, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not register handler to server")]
    Register,
    #[error("Used method not implemented")]
    Undefined,
    #[error("Could not start the server")]
    Start,
}
pub struct Handler {
    pub method: Method,
    pub uri: String,
    pub handler: Box<dyn Fn(String) -> Response + Send>,
}
pub trait ServerResource {
    fn create_server(&mut self, handlers: Vec<Handler>) -> Result<(),Error>;
}
pub trait HasServer {
    type Server;
    fn take_server(&mut self, id: usize) -> Option<Self::Server>;
}
