use std::sync::{Arc, Mutex};
use embedded_svc::eth;
use embedded_svc::eth::Eth;
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::utils::anyerror::*;
use embedded_svc::wifi::*;

use esp_idf_svc::eth::*;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sntp;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;


use esp_idf_svc::httpd as idf;
use thiserror::Error;

use crate::web_protocol::{Command, CommandBuffer, parse_request};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't Register endpoint")]
    RegisterEndpoint(embedded_svc::httpd::Error),
    #[error("Couldn't start server")]
    ServerStart,
    #[error("Resource busy")]
    ResourceBusy,
    #[error("Can't push command to the buffer")]
    BufferFull,

}

pub fn create_server(command_buffer: CommandBuffer) -> Result<idf::Server, Error>{
    let server = idf::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Eddig is siman meg tudtam volna!".into())).map_err(|e|Error::RegisterEndpoint(e))?
        .at("/command")
        .put(move |rq| {handle_command(rq,command_buffer.clone()).into()}).map_err(|e|Error::RegisterEndpoint(e))?
        .at("/bar")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        }).map_err(|e|Error::RegisterEndpoint(e))?;

    Ok(server.start(&Default::default()).map_err(|_|Error::ServerStart)?)
}
fn push_command(command: Command, command_buffer: CommandBuffer) ->Result<(), Error>{
    Ok(command_buffer.lock().map_err(|_|Error::ResourceBusy)?.try_push(command).map_err(|_| Error::BufferFull)?)
}
fn handle_command(request: Request, command_buffer: CommandBuffer) -> Response{
    match parse_request(request){
        Ok(command) => {
                match push_command(command, command_buffer) {
                    Ok(_) => {
                        Response::new(200)
                    }
                    Err(_) => {
                        Response::new(409)
                    }
                }
        }
        Err(error) => {
            Response::new(500)
        }
    }
}