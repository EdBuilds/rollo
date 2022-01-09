use std::borrow::{Borrow, BorrowMut};
use embedded_svc::http::client::{Client, Request, Response};
use embedded_svc::http::Status;
use embedded_svc::io::Read;
use esp_idf_svc::http::client::*;
use bal::client::{ClientResource, Error};
use bal::networking_types::{Method};
use crate::ClientContainer;

impl ClientResource for ClientContainer{
    fn make_request(&mut self, request: bal::networking_types::Request) -> Result<bal::networking_types::Response, Error> {
        let mut client = EspHttpClient::new_default().map_err(|_|Error::ClientCreate)?;
        let method = match request.method {
            Method::Get => {embedded_svc::http::Method::Get}
            Method::Put => {embedded_svc::http::Method::Put}
            Method::Post => {embedded_svc::http::Method::Post}
            Method::Delete => {embedded_svc::http::Method::Delete}
        };
        ;
        let rq = client.request(method, request.url.as_str()).map_err(|_|Error::ReqCreate)?;
        let rsp = rq.submit().map_err(|_|Error::ReqDisp)?;
        //todo this is nasty, but I don't know how does the esp reader work
        let mut buffer:[u8;500] = [0;500];
        rsp.reader().do_read(buffer.borrow_mut()).map_err(|_|Error::RespRead)?;
        let status = match rsp.status() {
            200 => {bal::networking_types::Status::Ok}
            400 => {bal::networking_types::Status::BadRequest}
            500 => {bal::networking_types::Status::InternalServerError}
            (statcode) => {return Err(Error::UnknownStatus(statcode))}
        };
        Ok(bal::networking_types::Response{ status: status, body: String::from_utf8_lossy(buffer.borrow()).to_string() })
    }
}