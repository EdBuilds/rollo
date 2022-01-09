mod server;
mod client;

#[macro_use]
use std::convert::Infallible;
use std::ops::Sub;
use num_traits::ToPrimitive;
use stepper::embedded_time::duration::Nanoseconds;
use stepper::motion_control;
use bal::stepper::{HasStepper, MCDelay, StepperResource};
use bal::switch::SwitchResource;
use bal::wifi::{HasWifi, WifiResource};
use bal::server::{Error, ServerResource};
use embedded_hal_stable;
use embedded_svc::httpd::registry::MiddlewareRegistry;
use embedded_svc::wifi::{AccessPointInfo, ApStatus, Capability, ClientStatus, Configuration, Status};
use enumset::EnumSet;
use void::Void;
use log::{Record, Level, Metadata, LevelFilter};
use std::io;

pub use server::*;

struct SimpleLogger;

pub struct ServerContainer;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
static LOGGER: SimpleLogger = SimpleLogger;
pub struct DelayToTicks;
impl motion_control::DelayToTicks<MCDelay> for DelayToTicks {
    type Ticks = NanosecWrapper;
    // depends on your timer
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: MCDelay)
                      -> Result<Self::Ticks, Self::Error>
    {
        println!("Delay converted:{:#?} ->{:#?}", delay,delay *(1000000000));
        Ok(NanosecWrapper::from((delay *(1000000000)).to_i64().unwrap()))
    }
}

pub struct NanosecWrapper { ns: i64}
impl From<Nanoseconds> for NanosecWrapper{
    fn from(nans: Nanoseconds) -> Self {
        NanosecWrapper{ns: nans.0 as i64}
    }
}
impl From<i64> for NanosecWrapper{
    fn from(int: i64) -> Self {
        NanosecWrapper{ns: int}
    }
}
impl Sub for NanosecWrapper {
    type Output = NanosecWrapper;

    fn sub(self, rhs: Self) -> Self::Output {
        NanosecWrapper{ns: (self.ns - rhs.ns)}
    }
}

pub struct StepperContainer {
    name: &'static str
}
impl stepper::traits::MotionControl for StepperContainer {
    type Velocity = MCDelay;
    type Error = stepper::motion_control::Error<(), (), Void, Infallible, Infallible>;

    fn move_to_position(&mut self, max_velocity: Self::Velocity, target_step: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset_position(&mut self, step: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update(&mut self) -> Result<bool, Self::Error> {
        println!("update called for,{}. Enter bool(y/n):",self.name);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        match input.as_str().chars().nth(0).expect("no input given") {
            'y' => {Ok(true)}
            'n' => {Ok(false)}
            _ => {
                println!("Invalid parameter given, returning False");
                Ok(false)
            }
        }
    }
}

pub struct SwitchContainer {
    name: &'static str
}
impl bal::switch::SwitchResource for SwitchContainer {
    fn is_high(&mut self) -> Result<bool, Infallible> {
        println!("Is High called for,{}. Enter bool(y/n):",self.name);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        match input.as_str().chars().nth(0).expect("no input given") {
            'y' => {Ok(true)}
            'n' => {Ok(false)}
            _ => {
                println!("Invalid parameter given, returning False");
                Ok(false)
            }
        }

    }

    fn is_low(&mut self) -> Result<bool, Infallible> {

        println!("is_low called for,{}. Enter bool(y/n):",self.name);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        match input.as_str().chars().nth(0).expect("no input given") {
            'y' => {Ok(true)}
            'n' => {Ok(false)}
            _ => {
                println!("Invalid parameter given, returning False");
                Ok(false)
            }
        }
    }
}

pub struct WifiContainer;
impl embedded_svc::wifi::Wifi for WifiContainer {
    type Error = bal::wifi::Error;

    fn get_capabilities(&self) -> Result<EnumSet<Capability>, Self::Error> {
        Err(bal::wifi::Error::Undefined)
    }

    fn get_status(&self) -> Status {
        Status{ 0: ClientStatus::Stopped, 1: ApStatus::Stopped }
    }

    fn scan(&mut self) -> Result<Vec<AccessPointInfo>, Self::Error> {
        Err(bal::wifi::Error::Undefined)
    }

    fn get_configuration(&self) -> Result<Configuration, Self::Error> {
        Err(bal::wifi::Error::Undefined)
    }

    fn set_configuration(&mut self, conf: &Configuration) -> Result<(), Self::Error> {
        Err(bal::wifi::Error::Undefined)
    }
}
pub struct ClientContainer;
pub struct BoardResources{
    pub steppers: [Option<StepperContainer>;2],
    pub switches: [Option<SwitchContainer>;2],
    pub wifis:    [Option<WifiContainer>;1],
    pub servers:  [Option<ServerContainer>;1],
    pub clients:  [Option<ClientContainer>;1],
}

pub struct BoardResourceBuilder {}

impl<'a> BoardResourceBuilder{
    pub fn resolve(target_accel: MCDelay) -> Option<BoardResources> {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Debug));
        Some(BoardResources{
            steppers: [Some(StepperContainer{name: "axis a stepper"}),Some(StepperContainer{name: "axis b stepper"})],
            switches: [Some(SwitchContainer{name: "axis a limit switch"}), Some(SwitchContainer{name: "axis b limit switch"})],
            wifis: [Some(WifiContainer)],
            servers: [Some(ServerContainer)],
            clients: [Some(ClientContainer)],
        })
    }
}
//impl<'a> HasWifi for BoardResources {
//    type Error = EspError;
//
//    fn take_wifi_resource(&mut self, id: usize) -> &mut dyn WifiResource {
//        self.wifi.take().unwrap().borrow_mut()
//    }
//}
//
//impl<'a> HasStepper for BoardResources {
//    fn take_stepper_resource(&mut self, id: usize) -> &mut dyn StepperResource {
//        self.axis_a_res.stepper_res.deref_mut()
//    }
//}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
