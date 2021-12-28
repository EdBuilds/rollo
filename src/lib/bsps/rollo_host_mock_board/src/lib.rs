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

pub struct MockStepper;
impl stepper::traits::MotionControl for MockStepper {
    type Velocity = MCDelay;
    type Error = stepper::motion_control::Error<(), (), Void, Infallible, Infallible>;

    fn move_to_position(&mut self, max_velocity: Self::Velocity, target_step: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset_position(&mut self, step: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update(&mut self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct MockSwitch;
impl embedded_hal_stable::digital::v2::InputPin for MockSwitch {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

pub struct MockWifi;
impl embedded_svc::wifi::Wifi for MockWifi {
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

pub struct MockServer;
impl ServerResource for MockServer {
    fn create_server(&mut self, registry: MiddlewareRegistry) -> Result<(), Error> {
        Ok(())
    }
}
pub struct BoardResources {
    pub stepper_a: StepperContainer,
    pub stepper_b: StepperContainer,
    pub limit_sw_a: InputPinContainer,
    pub limit_sw_b: InputPinContainer,
    pub wifi: WifiContainer,
    pub server: ServerContainer
}

#[macro_export]
macro_rules! polymorphic_enum {
    ($name:ident $macro1:ident, $($variant:ident($type:path),)*) => {
        pub enum $name { $($variant($type)),* }
        #[macro_export]
        macro_rules! $macro1 {
            ($on:expr, |$with:ident| $body:block) => {
                match $on {
                    $($name::$variant($with) => $body )*
                }
            }
        }
    }
}

polymorphic_enum! {
    WifiContainer use_wifi,
    WifiM(MockWifi),
}
polymorphic_enum! {
    ServerContainer use_server,
    ServerM(MockServer),
}
polymorphic_enum! {
    InputPinContainer use_input_pin,
    PinMock(MockSwitch),
}

polymorphic_enum! {
    StepperContainer use_stepper,
    StepperMock(MockStepper),
}


impl embedded_svc::wifi::Wifi for WifiContainer {
    type Error = bal::wifi::Error;

    fn get_capabilities(&self) -> Result<EnumSet<Capability>, Self::Error> {
        use_wifi!(&self, |s| {s.get_capabilities()})
    }

    fn get_status(&self) -> Status {
        use_wifi!(&self, |s| {s.get_status()})
    }

    fn scan(&mut self) -> Result<Vec<AccessPointInfo>, Self::Error> {
        use_wifi!(self, |s| {s.scan()})
    }

    fn get_configuration(&self) -> Result<Configuration, Self::Error> {
        use_wifi!(&self, |s| {s.get_configuration()})
    }

    fn set_configuration(&mut self, conf: &Configuration) -> Result<(), Self::Error> {
        use_wifi!(self, |s| {s.set_configuration(conf)})
    }
}
pub struct BoardResourceBuilder {
}

impl<'a> BoardResourceBuilder{
    pub fn resolve(target_accel: MCDelay) -> Option<BoardResources> {
        Some(BoardResources{
            stepper_a: StepperContainer::StepperMock(MockStepper),
            stepper_b: StepperContainer::StepperMock(MockStepper),
            limit_sw_a: InputPinContainer::PinMock(MockSwitch),
            limit_sw_b: InputPinContainer::PinMock(MockSwitch),
            wifi: WifiContainer::WifiM(MockWifi),
            server: ServerContainer::ServerM(MockServer)
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
