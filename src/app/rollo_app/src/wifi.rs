use std::sync::Arc;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::wifi::{AccessPointConfiguration, ApIpStatus, ApStatus, ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration, Status, Wifi};
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::*;
use ::log::info;
use thiserror::Error;
use crate::board_support::WifiResource;
use crate::wifi::Error::HwSetup;

const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

#[derive(Error, Debug)]
pub enum Error {
    #[error("Hardware setup error")]
    HwSetup,
    #[error("Scanning wifi failed")]
    WifiScan,
    #[error("Could not ping")]
    Ping,
    #[error("Timeout during pinging")]
    PingTimeout,
    #[error("Configuration error")]
    Config,
    #[error("Could not connect to access point. Status:`{0}`")]
    ConnectionStatus(String),
}

#[allow(dead_code)]
pub fn connect_wifi(wifi_res: &mut WifiResource
) -> Result<(), Error> {

    info!("Wifi created, about to scan");

    let ap_infos = use_wifi!(wifi_res, |wifi| {wifi.scan().map_err(|_| Error::WifiScan)?});

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    use_wifi!(wifi_res, |wifi| {wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    )).map_err(|_|Error::Config)?;});

    info!("Wifi configuration set, about to get status");

    let status = use_wifi!(wifi_res, |wifi| {wifi.get_status()});

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");

        ping(&ip_settings)?;
    } else {
        return Err(Error::ConnectionStatus(format!("{:#?}",status)))
    }

    Ok(())
}

fn ping(ip_settings: &ipv4::ClientSettings) -> Result<(),Error> {
    info!("About to do some pings for {:?}", ip_settings);
    let ping_summary =
        ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default()).map_err(|_|Error::Ping)?;
    if ping_summary.transmitted != ping_summary.received {
        return Err(Error::PingTimeout)
    }

    info!("Pinging done");

    Ok(())
}
