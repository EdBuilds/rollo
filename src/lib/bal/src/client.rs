use crate::networking_types::{Request, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client Creation error")]
    ClientCreate,
    #[error("Request creation error")]
    ReqCreate,
    #[error("Request dispatch error")]
    ReqDisp,
    #[error("Response reading error")]
    RespRead,
    #[error("Unknown status code:{0}")]
    UnknownStatus(u16),
}
pub trait ClientResource {
    fn make_request(&mut self, request: Request) -> Result<Response, Error>;
}