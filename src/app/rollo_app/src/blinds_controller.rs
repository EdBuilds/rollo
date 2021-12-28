#[macro_use]
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};
use arrayvec::ArrayVec;
use thiserror::Error;
use crate::{Axis};
use bal::stepper::MCDelay;
use bal::stepper::StepperResource;
use bal::wifi::{HasWifi, WifiResource};
use crate::web_protocol::Command;
use crate::web_server::create_server;
use crate::web_server;
use core::option::Option;
use std::marker::PhantomData;
use bal::server::ServerResource;
use board_support::*;
use board_support::wifi::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't connect to wifi")]
    WifiConnection,
    #[error("Couldn't create web server: {0}")]
    ServerCreation(web_server::Error),

}

pub struct BlindsController <'a>{
    commands_buffer: Arc<Mutex<ArrayVec<Command,2>>>,
    _phantom_data: PhantomData<&'a u32>,
    axis_angle: Axis<'a>,
    axis_opening: Axis<'a>,
    wifi_res: &'a mut dyn WifiResource,
    server: &'a mut dyn ServerResource,
}
impl<'a> BlindsController<'a>{
    pub fn new(board_res: &'a mut BoardResources) -> BlindsController<'a>{
        BlindsController{
            commands_buffer: Arc::new(Mutex::new(ArrayVec::new())),
            axis_angle: Axis::new(use_stepper!(&mut board_res.stepper_a,|stp|{stp.borrow_mut()}),
                                  use_input_pin!(&mut board_res.limit_sw_a,|sw|{sw.borrow_mut()}),
                                  1000,
                                  MCDelay::from_num(0.01)),
            axis_opening: Axis::new(use_stepper!(&mut board_res.stepper_b,|stp|{stp.borrow_mut()}),
                                  use_input_pin!(&mut board_res.limit_sw_b,|sw|{sw.borrow_mut()}),
                                  1000,
                                  MCDelay::from_num(0.01)),
            wifi_res: &mut board_res.wifi,
            server: use_server!(&mut board_res.server, |server|{server.borrow_mut()}),
            _phantom_data: Default::default(),
        }
    }
    pub fn start(&mut self) -> Result<(), Error>{
        connect_wifi(self.wifi_res.borrow_mut()).map_err(|_|Error::WifiConnection)?;
        create_server(self.server.borrow_mut(), self.commands_buffer.clone()).map_err(|e|Error::ServerCreation(e))?;
        Ok(())

    }
}