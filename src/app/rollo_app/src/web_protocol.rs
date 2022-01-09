use std::borrow::{Borrow, BorrowMut};
use std::io::Read;
use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use embedded_svc::httpd::*;
use log::{info, warn};
use queue::Queue;
use thiserror::Error;
#[derive(Serialize, Deserialize, Clone)]
pub struct Target {
    pub speed: f32,
    pub position: f32,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct MoveCommand {
    pub lateral: Target,
    pub angle: Target,
}
#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    Scheduler(SchedulerCommand),
    Controller(ControllerCommand),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ControllerCommand {
    Move(MoveCommand),
    Open,
    Close,
    Home,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SchedulerCommand {
    Start,
    Stop,
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


pub fn parse_request(rq: String) -> Result<Command, Error>{
    let parse_result = serde_json::from_str(rq.as_str()).map_err(|_| Error::Deserialize);
    if let Err(error) = parse_result.borrow(){
        println!("Could not serialize incoming data: {:#?}", error);
        println!("Possible json variations: {:#?}", [serde_json::ser::to_string(Command::Controller(ControllerCommand::Home).borrow())]);
    }
    parse_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_serialize() {
        println!("{:#?}",serde_json::ser::to_string(Command::Controller(ControllerCommand::Move(MoveCommand{ lateral: Target { speed: 10.0, position: 10.0 }, angle: Target { speed: 10.0, position: 0.0 } })).borrow()));
    }
}