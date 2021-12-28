use std::sync::Arc;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::wifi::{AccessPointConfiguration, ApIpStatus, ApStatus, ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration, Status, Wifi};
use thiserror::Error;
use bal::wifi::WifiResource;
use crate::wifi::Error::HwSetup;

const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

#[derive(Error, Debug)]
pub enum Error {
    #[error("Hardware setup error")]
    HwSetup,
    #[error("Scanning wifi failed")]
    WifiScan,
    #[error("Could not ping: {0}")]
    Ping(u32),
    #[error("Configuration error")]
    Config,
    #[error("Could not connect to access point. Status:`{0}`")]
    ConnectionStatus(String),
}

pub fn connect_wifi(wifi_res: &mut dyn WifiResource
) -> Result<(), Error> {

    let ap_infos = wifi_res.scan().map_err(|_| Error::WifiScan)?;
    //let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    //let channel = if let Some(ours) = ours {
    //    Some(ours.channel)
    //} else {
    //    None
    //};

    //wifi_res.set_configuration(&Configuration::Mixed(
    //    ClientConfiguration {
    //        ssid: SSID.into(),
    //        password: PASS.into(),
    //        channel,
    //        ..Default::default()
    //    },
    //    AccessPointConfiguration {
    //        ssid: "aptest".into(),
    //        channel: channel.unwrap_or(1),
    //        ..Default::default()
    //    },
    //)).map_err(|_|Error::Config)?;

    //let status = wifi_res.get_status();

    //if let Status(
    //    ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
    //    ApStatus::Started(ApIpStatus::Done),
    //) = status
    //{
    //    //ping(&ip_settings).map_err(|e|Error::Ping(e))?;
    //} else {
    //    return Err(Error::ConnectionStatus(format!("{:#?}",status)))
    //}

    Ok(())
}

