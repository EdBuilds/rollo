use std::borrow::{Borrow, BorrowMut};
use std::io::Read;
use std::sync::{Arc, Mutex};
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use embedded_svc::httpd::*;
use thiserror::Error;
#[derive(Serialize, Deserialize)]
pub struct Target {
    velocity: f32,
    position: f32,
}
#[derive(Serialize, Deserialize)]
pub struct MoveCommand {
    opening: Target,
    angle: Target,
}
#[derive(Serialize, Deserialize)]
pub enum Command {
    Move(MoveCommand),
    Home,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't parse the request")]
    RequestParsing,
    #[error("Couldn't Deserialize the request")]
    Deserialize,
    #[error("Stepper driver encountered an error")]
    Driver,
    #[error("Motion controller calculation error")]
    MotionControl,

}
pub type CommandBuffer = Arc<Mutex<ArrayVec<Command,2>>>;

pub fn parse_request(mut rq:Request) -> Result<Command, Error>{
    let comm:Command = Command::Move(MoveCommand{ opening: Target { velocity: 0.0, position: 0.0 }, angle: Target { velocity: 0.0, position: 0.0 } });
    let homecom = Command::Home;
    println!("{:#?}",serde_json::to_string(homecom.borrow()));
    let mut buffer = String::new();
    rq.read_to_string(buffer.borrow_mut()).map_err(|_|Error::RequestParsing)?;
    let parsed_command: Command = serde_json::from_str(&*buffer).map_err(|_| Error::Deserialize)?;
    Ok(parsed_command)
}