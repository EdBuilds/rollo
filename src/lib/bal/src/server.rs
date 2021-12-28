use embedded_svc::httpd::registry::Registry;
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

pub trait ServerResource {
    fn create_server(&mut self, registry: embedded_svc::httpd::registry::MiddlewareRegistry) -> Result<(),Error>;
}
pub trait HasServer {
    type Server;
    fn take_server(&mut self, id: usize) -> Option<Self::Server>;
}
