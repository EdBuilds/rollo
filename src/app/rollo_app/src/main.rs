#[macro_use]
mod axis;
mod web_server;
mod web_protocol;
mod blinds_controller;

use axis::*;
use std::{thread, time::Duration};
use std::borrow::BorrowMut;
use fixed::traits::Fixed;
use nb::block;
use num_traits::cast::ToPrimitive;
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

use thiserror::Error;
use bal::stepper::MCDelay;
use bal::stepper::HasStepper;
use bal::wifi::HasWifi;
use bal::wifi::WifiResource;
use bal::stepper::StepperResource;
use board_support::*;
#[cfg(target_os = "espidf")]
use esp_idf_sys::link_patches;

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
    #[cfg(target_os = "espidf")]
    link_patches();
    let target_accel = MCDelay::from_num(1f32); // steps / tick^2; 1000 steps / s^2
    let mut board_resurces = BoardResourceBuilder::resolve(target_accel).unwrap();
    board_resurces.wifi.scan();
    let mut b_c = BlindsController::new(&mut board_resurces);
    println!("{:#?}",b_c.start());

    for count in 1..1000 {
        println!("Hello world!{}", count);
        thread::sleep(Duration::from_secs(1));
    }
}
