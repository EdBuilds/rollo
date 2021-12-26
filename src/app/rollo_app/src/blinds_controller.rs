use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};
use arrayvec::ArrayVec;
use thiserror::Error;
use crate::{Axis, MCDelay};
use crate::board_support::{BoardResources, WifiResource};
use crate::web_protocol::Command;
use crate::web_server::create_server;
use crate::wifi::connect_wifi;
use crate::wifi;
use crate::web_server;
use core::option::Option;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::httpd::Server;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't connect to wifi: {0}")]
    WifiConnection(wifi::Error),
    #[error("Couldn't create web server: {0}")]
    ServerCreation(web_server::Error),

}

pub struct BlindsController {
    commands_buffer: Arc<Mutex<ArrayVec<Command,2>>>,
    axis_angle: Axis,
    axis_opening: Axis,
    wifi_res: WifiResource,
    server: Option<Server>,
}
impl BlindsController{
    pub fn new(board_res: BoardResources) -> BlindsController{
        BlindsController{
            commands_buffer: Arc::new(Mutex::new(ArrayVec::new())),
            axis_angle: Axis::new(board_res.axis_a_res, 1000, MCDelay::from_num(0.01)),
            axis_opening: Axis::new(board_res.axis_b_res, 2000, MCDelay::from_num(0.001)),
            wifi_res: board_res.wifi,
            server: None
        }
    }
    pub fn start(&mut self) -> Result<(), Error>{
        connect_wifi(self.wifi_res.borrow_mut()).map_err(|e|Error::WifiConnection(e))?;
        self.server = Some(create_server(self.commands_buffer.clone()).map_err(|e|Error::ServerCreation(e))?);
        Ok(())

    }
}