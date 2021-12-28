use embedded_svc;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Used method not implemented")]
    Undefined,
}
pub trait WifiResource = embedded_svc::wifi::Wifi<Error = Error>;
pub trait HasWifi {
    type Error;
    fn take_wifi_resource(&mut self, id: usize) -> &mut dyn WifiResource;
}