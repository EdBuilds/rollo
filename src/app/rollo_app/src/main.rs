#![feature(future_poll_fn)]
#[macro_use]
mod axis;

mod blinds_controller;
mod user_settings;
mod scheduler;
mod network;
mod web_protocol;
mod astronomy;
mod astronomy_api;


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
use bal::wifi::HasWifi;
use bal::wifi::WifiResource;
use bal::stepper::StepperResource;
use board_support::*;
#[cfg(target_os = "espidf")]
use esp_idf_sys;
use log::*;
use crate::network::Network;
use crate::scheduler::Scheduler;
use crate::user_settings::{BlindsUserSettings, UserSettings};

use crate::web_protocol::parse_request;

#[feature(backtrace)]
fn main() {
    println!("Entry!");
    #[cfg(target_os = "espidf")]
        {
            esp_idf_sys::link_patches();
        }

    let target_accel = MCDelay::from_num(1000f32); // steps / tick^2; 1000 steps / s^2
    let mut board_resurces = BoardResourceBuilder::resolve(target_accel).unwrap();
    println!("Board resources initialized!");
    let mut network = Network::new(board_resurces.borrow_mut());
    println!("Network created");
    match network.create_server(){
        Ok(thread_signals) => {
            let mut b_c = BlindsController::new(board_resurces.borrow_mut(), UserSettings::default(), thread_signals.blinds_ctrl_signal.clone());
            let mut scheduler = Scheduler::new(board_resurces.borrow_mut(),
                                               thread_signals.scheduler_signal.clone(),
                                               thread_signals.blinds_ctrl_signal.clone());
            thread::Builder::new().stack_size(5000).name("Blinds controller".to_string()).spawn(move || {
                    b_c.start();
            });
            thread::Builder::new().stack_size(15000).name("Scheduler".to_string()).spawn(move || {
                scheduler.start();
            });

        }
        Err(error) => {
            println!("Couldn't start server: {}", error);

        }
    };
    println!("Blinds controller size: {}", std::mem::size_of::<BlindsController>());
    loop {
        thread::sleep(Duration::from_millis(500));
    }

}
