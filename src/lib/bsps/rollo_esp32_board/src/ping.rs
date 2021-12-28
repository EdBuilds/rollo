use embedded_svc::ipv4;
use esp_idf_svc::ping;
use embedded_svc::ping::Ping;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not Set up ping")]
    SetUp,
    #[error("Timeout during pinging")]
    Timeout,
}

pub fn ping(ip_settings: &ipv4::ClientSettings) -> Result<(),Error> {
    let ping_summary =
        ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default()).map_err(|_|Error::SetUp)?;
    if ping_summary.transmitted != ping_summary.received {
        return Err(Error::Timeout)
    }


    Ok(())
}
