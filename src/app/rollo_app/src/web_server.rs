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

use thiserror::Error;
use bal::server::ServerResource;

use crate::web_protocol::{Command, CommandBuffer, parse_request};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't register endpoint")]
    RegisterEndpoint,
    #[error("Couldn't start web server: {0}")]
    ServerCreation(bal::server::Error),
    #[error("Resource busy")]
    ResourceBusy,
    #[error("Can't push command to the buffer")]
    BufferFull,

}

pub fn create_server(server_res: & mut dyn ServerResource, command_buffer: CommandBuffer) -> Result<(), Error>{
    let registry = embedded_svc::httpd::registry::MiddlewareRegistry::new()
        .at("/")
        .get(|_| Ok("Eddig is siman meg tudtam volna!".into())).map_err(|_|Error::RegisterEndpoint)?
        .at("/command")
        .put(move |rq| {handle_command(rq,command_buffer.clone()).into()}).map_err(|_|Error::RegisterEndpoint)?
        .at("/bar")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        }).map_err(|_|Error::RegisterEndpoint)?;
    Ok(server_res.create_server(registry).map_err(|e|Error::ServerCreation(e))?)

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