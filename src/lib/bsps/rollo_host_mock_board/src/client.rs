use std::borrow::BorrowMut;
use std::io::Read;
use std::num::NonZeroU16;
use bal::client::{ClientResource, Error};
use bal::networking_types::{Method, Request, Response, Status};
use crate::ClientContainer;
use reqwest;
use reqwest::StatusCode;

impl ClientResource for ClientContainer {
    fn make_request(&mut self, request: Request) -> Result<Response, Error> {
        match request.method {
            Method::Get => {
                let mut resp = reqwest::blocking::get(request.url).map_err(|_| Error::ReqDisp)?;
                let status = match resp.status().as_u16() {
                    200 => {Status::Ok}
                    400 => {Status::BadRequest}
                    500 => {Status::InternalServerError}
                    code => {return Err(Error::UnknownStatus(code))}
                };
                let mut buffer = String::new();
                resp.read_to_string(buffer.borrow_mut()).map_err(|_|Error::RespRead);
                Ok(Response{ status: status, body: buffer })
            }
            _ => {
                Err(Error::ReqCreate)
            }
        }

    }
}