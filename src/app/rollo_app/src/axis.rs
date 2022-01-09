use core::fmt;
use core::task::Poll;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::convert::Infallible;
use std::fmt::Formatter;
use std::future::Future;
use std::io;
use std::ops::Mul;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Context;
use log::info;
use bal::stepper::StepperResource;
use bal::switch::SwitchResource;
use crate::user_settings::AxisUserSettings;
use thiserror::Error;
use void::Void;
use bal::stepper::MCDelay;
use board_support::{StepperContainer, SwitchContainer};
//remove

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't find home before reaching the limit")]
    HomeLimitReached,
    #[error("Stepper driver encountered an error")]
    Driver,
    #[error("Failed to read limit switch")]
    Swtich,
    #[error("Motion controller calculation error")]
    MotionControl,
    #[error("Stepper unexpectedly reached it's limit")]
    LimitTrigger,
    #[error("Stepper Axis hasn't been homed before movement")]
    NotHomed,
    #[error("Invalid input given")]
    InputError,
    #[error("The requested motion is outside of the axis range")]
    OutOfRange,
    #[error("Something that's not supposed to fail failed")]
    Infallible,
}

pub struct Axis
{
    stepper: StepperContainer,
    limit_switch: SwitchContainer,
    settings: AxisUserSettings,
    homed: bool,
}

impl Axis
{
    pub fn new(stepper: StepperContainer, switch: SwitchContainer, settings: AxisUserSettings) -> Axis
    {
        Axis { stepper, limit_switch: switch, settings, homed: true}
    }
    pub fn home(&mut self) -> Result<(), Error> {

        self.stepper.move_to_position(self.settings.homing_speed, -self.settings.range).map_err(|_|Error::MotionControl)?;
        loop {
            if self.stepper.update().map_err(|_| Error::Driver)? {
                if self.limit_switch.is_low().map_err(|_| Error::Swtich)? {
                    self.stepper.reset_position(self.settings.home_position).map_err(|_| Error::MotionControl)?;
                    self.homed = true;
                    break;
                }
            } else {
                return Err(Error::HomeLimitReached);
            }
        }
        self.stepper.move_to_position(self.settings.homing_speed, 0).map_err(|_|Error::MotionControl)?;
        loop {
            if !self.stepper.update().map_err(|_| Error::Driver)? {
                return Ok(())
            }

        }
    }
pub fn move_to(&mut self, step: i32, speed_prec: f32) -> Result<(), Error>{
    let velocity = self.settings.max_speed.mul(MCDelay::from_num(speed_prec));
    if step > self.settings.range || step < 0 {
        Err(Error::OutOfRange)?;
    }
    if !self.homed {
        Err(Error::NotHomed)?;
    }
    self.stepper.move_to_position(velocity, step);
    loop {
        if self.stepper.update().map_err(|_| Error::Driver)? {
            if let Ok(switch_active) = self.limit_switch.is_high() {
                if switch_active {
                    Err(Error::LimitTrigger)?;
                }
            }
        } else {
            break;
            }
        }
        Ok(())
    }
    pub fn update_settings(&mut self, settings: AxisUserSettings){

    }
}
