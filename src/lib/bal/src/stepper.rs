use std::convert::Infallible;
use stepper;
use fixed;
use void::Void;
pub type MCDelay = fixed::FixedI128<fixed::types::extra::U64>;
pub trait StepperResource = stepper::traits::MotionControl<Velocity = MCDelay, Error = stepper::motion_control::Error<(), (), Void, Infallible, Infallible>>;
pub trait HasStepper{
    type Resource;
    fn take_stepper(&mut self, id: usize) -> Option<Self::Resource>;
}
