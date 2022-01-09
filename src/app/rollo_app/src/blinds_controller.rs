#[macro_use]
use std::borrow::BorrowMut;
use std::sync::{Arc};
use arrayvec::ArrayVec;
use thiserror::Error;
use crate::{Axis, network};
use bal::stepper::MCDelay;
use bal::stepper::StepperResource;
use bal::wifi::{HasWifi, WifiResource};
use crate::web_protocol::{Command, ControllerCommand, MoveCommand};
use core::option::Option;
use std::marker::PhantomData;
use std::ops::Deref;
use log::{debug, error, info, warn};
use bal::server::ServerResource;
use board_support::*;
use board_support::wifi::*;
use queue;
use queue::Queue;
use crate::user_settings::{BlindsUserSettings, UserSettings};
use crate::axis;
use std::time::Duration;
use std::sync::Mutex;
use std::thread;
use bal::stepper::HasStepper;
use crate::network::ThreadSignal;


#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't set the thread id for the comms channel:{0}")]
    ThreadSet(network::Error),
    #[error("Couldn't move to the desired location: {0}")]
    MovementError(axis::Error),
    #[error("Couldn't home axis: {0}")]
    HomingError(axis::Error),

}
pub struct BlindsController
{
    thread_signal: ThreadSignal<ControllerCommand>,
    axis_angle: Axis,
    axis_lateral: Axis,
    settings: BlindsUserSettings,
}
impl BlindsController
{
    pub fn new(board_res: &mut BoardResources, settings: UserSettings, thread_signal: ThreadSignal<ControllerCommand>) -> BlindsController{
        BlindsController{
            thread_signal,
            axis_angle: Axis::new(board_res.steppers.get_mut(0).unwrap().take().unwrap(),
                                  board_res.switches.get_mut(0).unwrap().take().unwrap(),
                                    settings.axis_angle_settings
            ),
            axis_lateral: Axis::new(board_res.steppers.get_mut(1).unwrap().take().unwrap(),
                                    board_res.switches.get_mut(1).unwrap().take().unwrap(),
                                    settings.axis_open_settings
            ),
            settings: settings.blinds_settings
        }
    }
    fn move_lateral(&mut self, step: i32, speed: f32)->Result<(), Error> {

        self.axis_angle.move_to(self.settings.angle_open, speed).map_err(|e|Error::MovementError(e))?;
        self.axis_lateral.move_to(step, speed).map_err(|e|Error::MovementError(e))

    }
    fn process_move_command(&mut self, move_command:MoveCommand)->Result<(), Error> {
        self.move_lateral(move_command.lateral.position as i32, move_command.lateral.speed)?;
        self.axis_angle.move_to(move_command.angle.position as i32, move_command.lateral.speed).map_err(|e|Error::MovementError(e))

    }
    fn process_home_command(&mut self)->Result<(), Error> {

        self.axis_angle.home().map_err(|e|Error::HomingError(e))?;
        self.axis_lateral.home().map_err(|e|Error::HomingError(e))

    }
    fn command_executer(&mut self) {
        loop {
            thread::park();
            println!("thread woken");
            match self.thread_signal.pop_item() {
                Ok(command) => {
                match command {
                    ControllerCommand::Move(move_command) => {
                        info!("Executing move command...");
                        match self.process_move_command(move_command) {
                            Ok(_) => {
                            info!("Done!");}
                            Err(error) => {
                                warn!("Error during Execution of move command: {}", error);
                            }
                        }
                    }
                    ControllerCommand::Home => {
                        info!("Executing home command...");
                        match self.process_home_command() {
                            Ok(_) => {
                                info!("Done!");}
                            Err(error) => {
                                warn!("Error during Execution of home command: {}", error);
                            }
                        }
                    }
                    ControllerCommand::Open => {
                        info!("Opening Blinds!")
                        // todo Actually implement this
                    }
                    ControllerCommand::Close => {
                        info!("Closing Blinds!")
                        // todo Actually implement this
                    }
                }
                }
                    Err(error) => {
                        error!("Blinds controller coldn't get item from the queue:{}", error);
                    }
            }
            }
        }
    fn setup(&mut self) -> Result<(), Error>{
        self.thread_signal.set_thread().map_err(|e|Error::ThreadSet(e))?;
        Ok(())

    }
    pub fn start(&mut self) {
        match self.setup() {
            Ok(_) => {
                println!("Command executer started");
                self.command_executer();
            }
            Err(error) => {
                println!("Could not start program, due to : {:#?}", error);
            }
        }

    }
}