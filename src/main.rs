#[macro_use]
mod board_support;
mod axis;
mod wifi;
mod web_server;
mod blinds_controller;

use axis::*;
use esp_idf_svc::sysloop::*;
use std::{thread, time::Duration};
use std::borrow::BorrowMut;
use stepper::{motion_control, ramp_maker, Stepper};
use esp32_hal::gpio;
use esp32_hal::target::Peripherals;
use esp32_hal::units::Ticks;
use fixed::traits::Fixed;
use stepper::embedded_hal::prelude::_embedded_hal_timer_CountDown;
use nb::block;
use stepper::embedded_time::duration::Nanoseconds;
use stepper::motion_control::Error::Step;
use num_traits::cast::ToPrimitive;
use embedded_hal_stable::digital::v2::InputPin;
use embedded_time::duration::Microseconds;
use board_support::{BoardResourceBuilder, InputPinResource, TimerResource, NanosecWrapper, TimerWrapper, AxisResource, StepperResource, MCDelay};
use smol::future::block_on;
use smol::Executor;

fn main() -> anyhow::Result<()> {
    let target_accel = MCDelay::from_num(1f32); // steps / tick^2; 1000 steps / s^2
    let mut board_resurces = BoardResourceBuilder::resolve(target_accel).unwrap();
    let max_vel = MCDelay::from_num(10f32); // steps / tick^2; 1000 steps / s^2
    let ax_a = Axis::new(board_resurces.axis_a_res, 100, max_vel);
    let homed_ax_a_res = block_on(ax_a.home(-10, MCDelay::from_num(100f32)));
    println!("{:#?}", homed_ax_a_res);
    loop {
        println!("Hello world!");
        thread::sleep(Duration::from_secs(1));
    }
}
