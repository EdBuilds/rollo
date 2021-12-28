use core::fmt;
use core::task::Poll;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::convert::Infallible;
use std::fmt::Formatter;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Context;
use bal::stepper::StepperResource;
use bal::switch::SwitchResource;

use thiserror::Error;
use void::Void;
use bal::stepper::MCDelay;

//remove

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


pub struct Axis<'a> {
    stepper: &'a mut dyn StepperResource,
    limit_switch: &'a mut dyn SwitchResource,
    range: i32,
    velocity: MCDelay,
}

impl<'a> Axis<'a>
{
    pub fn new(stepper: &'a mut dyn StepperResource, switch: &'a mut dyn SwitchResource, range: i32, velocity: MCDelay) -> Axis<'a>{
        Axis { stepper, limit_switch: switch, range, velocity}
    }
    pub fn home(&mut self, home_step: i32, homing_speed: MCDelay) -> impl Future<Output = Result<(), Error>>+ '_ {
            HomeFuture::new(self, homing_speed, home_step)
    }
}
#[derive(Clone, Copy)]
enum HomingState {
    Start,
    Homing,
    Returning,
}
struct HomeFuture<'a>{
    motion_control: &'a mut dyn StepperResource,
    range: Option<i32>,
    limit_switch: &'a mut dyn SwitchResource,
    state: HomingState,
    homing_speed: MCDelay,
    home_step: i32,
    velocity: Option<MCDelay>,
}
impl HomeFuture<'_>
{
    fn new<'a>(axis: &'a mut Axis, homing_speed: MCDelay, home_step:i32) -> HomeFuture<'a>
    {
        HomeFuture{ motion_control: axis.stepper.borrow_mut(), range: Some(axis.range), limit_switch: axis.limit_switch.borrow_mut(), state: HomingState::Start, homing_speed , velocity: Some(axis.velocity), home_step}
    }
}
impl Future for HomeFuture<'_> {
    type Output = Result<(), Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let speed = self.homing_speed;
        let state = self.state;
        match state {
            HomingState::Start => {
                let range = self.range.unwrap();
                (*self.motion_control).move_to_position(speed, -range).map_err(|_|Error::MotionControl)?;
                    self.state = HomingState::Homing;
                cx.waker().wake_by_ref();
                Poll::Pending

            }
            HomingState::Homing => {
                if (*self.motion_control).update().map_err(|_|Error::Driver)?{
                        if let Ok(homed) = self.limit_switch.is_low(){
                            if homed{
                                let home_step = self.home_step;

                                (*self.motion_control).reset_position(-home_step).map_err(|_|Error::MotionControl)?;
                                (*self.motion_control).move_to_position(speed, 0).map_err(|_|Error::MotionControl)?;
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

                if (*self.motion_control).update().map_err(|_|Error::Driver)?{
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {

                    Poll::Ready(Ok(()))

                }
            }
        }

    }
}