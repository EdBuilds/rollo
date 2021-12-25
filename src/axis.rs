use core::fmt;
use core::task::Poll;
use std::cell::RefCell;
use std::convert::Infallible;
use std::fmt::Formatter;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Context;

use thiserror::Error;
use embedded_hal_stable::digital::v2::InputPin;
use stepper::MoveToFuture;
use stepper::util::ref_mut::RefMut;
use void::Void;

//remove
use crate::{AxisResource, InputPinResource, MCDelay, StepperResource};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't find home before reaching the limit")]
    HomeLimitReached,
    #[error("Stepper driver encountered an error")]
    Driver,
    #[error("Motion controller calculation error")]
    MotionControl,
    #[error("Something that's not supposed to fail failed")]
    Infallible,
}

pub struct HomedAxis {
    motion_control: Mutex<Box<dyn stepper::traits::MotionControl<Velocity = MCDelay, Error = stepper::motion_control::Error<(), (), Void, Infallible, Infallible>>>>,
    limit_switch: InputPinResource,
    range: i32,
    velocity: MCDelay,
}
impl fmt::Debug for HomedAxis {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("homed_axis").finish()

    }
}

pub struct Axis {
    stepper: StepperResource,
    limit_switch: InputPinResource,
    range: i32,
    velocity: MCDelay,
}

impl Axis
{
    pub fn new(axis_res:AxisResource, range: i32, velocity: MCDelay) -> Axis {
        Axis { stepper: axis_res.stepper_res, limit_switch: axis_res.limit_sw_pin, range, velocity }
    }
    pub fn home(mut self, home_step: i32, homing_speed: MCDelay) -> impl Future<Output = Result<HomedAxis, Error>> {
            HomeFuture::new(self, homing_speed, home_step)
    }
}
#[derive(Clone, Copy)]
enum HomingState {
    Start,
    Homing,
    Returning,
}
struct HomeFuture{
    motion_control: Option<Mutex<Box<dyn stepper::traits::MotionControl<Velocity = MCDelay, Error = stepper::motion_control::Error<(), (), Void, Infallible, Infallible>>>>>,
    range: Option<i32>,
    limit_switch: Option<InputPinResource>,
    state: HomingState,
    homing_speed: MCDelay,
    home_step: i32,
    velocity: Option<MCDelay>,
}
impl HomeFuture {
    fn new(mut axis: Axis, homing_speed: MCDelay, home_step:i32) -> HomeFuture
    {
        let what: Mutex<Box<dyn stepper::traits::MotionControl<Velocity = MCDelay, Error = stepper::motion_control::Error<_, _, _, _, _>>>> = use_stepper!( axis.stepper, |stp|{Mutex::new(Box::new(stp.release()))});
        HomeFuture{ motion_control: Some(what), range: Some(axis.range), limit_switch: Some(axis.limit_switch), state: HomingState::Start, homing_speed , velocity: Some(axis.velocity), home_step}
    }
}
impl Future for HomeFuture {
    type Output = Result<HomedAxis, Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let speed = self.homing_speed;
        let state = self.state;
        match state {
            HomingState::Start => {
                let range = self.range.unwrap();
                    self.motion_control.as_mut().unwrap().lock().unwrap().move_to_position(speed, -range).map_err(|_|Error::MotionControl)?;
                    self.state = HomingState::Homing;
                cx.waker().wake_by_ref();
                Poll::Pending

            }
            HomingState::Homing => {
                if self.motion_control.as_mut().unwrap().lock().unwrap().update().map_err(|_|Error::Driver)?{
                        if let Ok(homed) = use_input_pin!(self.limit_switch.as_mut().unwrap() ,|input_pin|{input_pin.is_low()}){
                            if homed{
                                let home_step = self.home_step;

                                self.motion_control.as_mut().unwrap().lock().unwrap().reset_position(-home_step).map_err(|_|Error::MotionControl)?;
                                self.motion_control.as_mut().unwrap().lock().unwrap().move_to_position(speed, 0).map_err(|_|Error::MotionControl)?;
                                self.state = HomingState::Returning;
                            }
                                cx.waker().wake_by_ref();
                                Poll::Pending
                        } else {
                            Poll::Ready(Err(Error::Infallible))
                        }
                    }
                    else {
                            Poll::Ready(Err(Error::HomeLimitReached))
                        }
                    }
            HomingState::Returning => {

                if self.motion_control.as_mut().unwrap().lock().unwrap().update().map_err(|_|Error::Driver)?{
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {

                    Poll::Ready(Ok(HomedAxis{
                        motion_control: self.motion_control.take().unwrap(),
                        limit_switch: self.limit_switch.take().unwrap(),
                        range: self.range.take().unwrap(),
                        velocity: self.velocity.take().unwrap(),
                    }))

                }
            }
        }

    }
}