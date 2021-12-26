#[macro_use]
mod board_support;
mod axis;
mod wifi;
mod web_server;
mod web_protocol;
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
use blinds_controller::BlindsController;
use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, time::*};

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

use crate::web_protocol::parse_request;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't Register endpoint")]
    RegisterEndpoint(embedded_svc::httpd::Error),
    #[error("Couldn't start server")]
    ServerStart,

}

#[feature(backtrace)]
fn main() {
    esp_idf_sys::link_patches();
    let target_accel = MCDelay::from_num(1f32); // steps / tick^2; 1000 steps / s^2
    let mut board_resurces = BoardResourceBuilder::resolve(target_accel).unwrap();
    let mut b_c = BlindsController::new(board_resurces);
    b_c.start().unwrap();

    for count in 1..1000 {
        println!("Hello world!{}", count);
        thread::sleep(Duration::from_secs(1));
    }
    drop(b_c);
}
