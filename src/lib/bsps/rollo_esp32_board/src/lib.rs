#[macro_use]
pub mod ping;
pub mod server;
pub mod client;

use esp_idf_sys;
use std::borrow::BorrowMut;
use std::convert::Infallible;
use std::fmt::Error;
use std::io::Read;
use std::marker::PhantomData;
use std::ops::{DerefMut, Mul, Sub};
use std::sync::Arc;
#[cfg(not(feature = "ulp"))]
use esp_idf_sys::EspMutex;
use stepper::drivers::drv8825;
use stepper::drivers::drv8825::DRV8825;
use stepper::traits::{EnableDirectionControl, EnableStepControl};
use embedded_hal_stable::digital::v2::{InputPin, OutputPin};
use embedded_hal_stable::timer::CountDown;
use embedded_svc::wifi::{AccessPointInfo, Capability, Configuration, Status};
use embedded_time::duration::Microseconds;
use embedded_time::rate::Microhertz;
use embedded_time::timer::param::None;
use esp32_hal::clock_control::{ClockControl, ClockControlConfig};
use esp32_hal::prelude::*;
use esp32_hal::gpio::*;
use esp32_hal::timer::*;
use esp32_hal::clock_control::*;
use esp32_hal::dport::Split;
use esp32_hal::target::{GPIO, Peripherals, TIMG0, TIMG1};
use esp32_hal::units;
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::EspError;
use stepper::{Direction, motion_control, ramp_maker, SignalError, Stepper};
use stepper::embedded_time::duration::Nanoseconds;
use stepper::motion_control::SoftwareMotionControl;
use stepper::ramp_maker::Trapezoidal;
use nb;
use crate::server::EspWebServer;
use void::Void;
use bal::stepper::{HasStepper, MCDelay, StepperResource};
use bal::switch::SwitchResource;
use bal::wifi::{HasWifi, WifiResource};
use bal::server::{Handler, ServerResource};
use bal::Takeable;
use fixed::prelude::*;
use num_traits::cast::ToPrimitive;
use enumset;
use enumset::EnumSet;
use esp_idf_svc::http::client::EspHttpClient;

pub struct DelayToTicks;
impl motion_control::DelayToTicks<MCDelay> for DelayToTicks {
    type Ticks = NanosecWrapper;
    // depends on your timer
    type Error = core::convert::Infallible;

    fn delay_to_ticks(&self, delay: MCDelay)
                      -> Result<Self::Ticks, Self::Error>
    {
        Ok(NanosecWrapper::from((delay *(1000000000)).to_i64().unwrap()))
    }
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
    Wifi0(EspWifi),
}
polymorphic_enum! {
    ServerContainer use_server,
    Server0(EspWebServer),
}

impl bal::server::ServerResource for ServerContainer {
    fn create_server(&mut self, handlers: Vec<Handler>) -> Result<(), bal::server::Error> {
       use_server!(self, |server| {server.create_server(handlers)})
    }
}
polymorphic_enum! {
    SwitchContainer use_switch,
    Gpio17InPD(Gpio21<Input<PullDown>>),
    Gpio18InPD(Gpio22<Input<PullDown>>),
}
impl bal::switch::SwitchResource for SwitchContainer {
    fn is_low(&mut self) -> Result<bool, Infallible> {
        use_switch!(self, |sw|{sw.is_low()})
    }

    fn is_high(&mut self) -> Result<bool, Infallible> {
        use_switch!(self, |sw|{sw.is_high()})
    }
}
polymorphic_enum! {
    TimerContainer use_timer,
    TimerG00(TimerWrapper<Timer<TIMG0, Timer0>>),
    TimerG01(TimerWrapper<Timer<TIMG0, Timer1>>),
}

polymorphic_enum! {
    StepperContainer use_stepper,
    stepper1(SoftwareMotionControl<DRV8825<(), (), (), OutputWrapper<Gpio13<Output<PushPull>>>, OutputWrapper<Gpio16<Output<PushPull>>>, OutputWrapper<Gpio17<Output<PushPull>>>, OutputWrapper<Gpio18<Output<PushPull>>>, OutputWrapper<Gpio2<Output<PushPull>>>, OutputWrapper<Gpio5<Output<PushPull>>>>, TimerWrapper<Timer<TIMG0, Timer0>>, Trapezoidal<MCDelay>, DelayToTicks>),
    stepper2(SoftwareMotionControl<DRV8825<(), (), (), (), (), (), (), OutputWrapper<Gpio4<Output<PushPull>>>, OutputWrapper<Gpio15<Output<PushPull>>>>, TimerWrapper<Timer<TIMG0, Timer1>>, Trapezoidal<MCDelay>, DelayToTicks>),
}
impl stepper::traits::MotionControl for StepperContainer {
    type Velocity = MCDelay;
    type Error = stepper::motion_control::Error<(), (), Void, Infallible, Infallible>;

    fn move_to_position(&mut self, max_velocity: Self::Velocity, target_step: i32) -> Result<(), Self::Error> {
        use_stepper!{self, |stp|{stp.move_to_position(max_velocity, target_step)}}
    }

    fn reset_position(&mut self, step: i32) -> Result<(), Self::Error> {
        use_stepper!{self, |stp|{stp.reset_position(step)}}
    }

    fn update(&mut self) -> Result<bool, Self::Error> {
        use_stepper!{self, |stp|{stp.update()}}
    }
}
impl embedded_svc::wifi::Wifi for WifiContainer {
    type Error = bal::wifi::Error;

    fn get_capabilities(&self) -> Result<EnumSet<Capability>, Self::Error> {
        use_wifi!(&self, |s| {s.get_capabilities().map_err(|_| bal::wifi::Error::Undefined)})
    }

    fn get_status(&self) -> Status {
        use_wifi!(&self, |s| {s.get_status()})
    }

    fn scan(&mut self) -> Result<Vec<AccessPointInfo>, Self::Error> {
        println!("11");
        use_wifi!(self, |s| {s.scan().map_err(|_| bal::wifi::Error::Undefined)})
    }

    fn get_configuration(&self) -> Result<Configuration, Self::Error> {
        use_wifi!(&self, |s| {s.get_configuration().map_err(|_| bal::wifi::Error::Undefined)})
    }

    fn set_configuration(&mut self, conf: &Configuration) -> Result<(), Self::Error> {
        use_wifi!(self, |s| {s.set_configuration(conf).map_err(|_| bal::wifi::Error::Undefined)})
    }
}

pub struct ClientContainer();
pub struct BoardResources{
    pub steppers: [Option<StepperContainer>;2],
    pub switches: [Option<SwitchContainer>;2],
    pub wifis:    [Option<WifiContainer>;1],
    pub servers:  [Option<ServerContainer>;1],
    pub clients:  [Option<ClientContainer>;1],
}

pub struct BoardResourceBuilder {
}

pub struct OutputWrapper<T>{
    pin: T
}
impl<T: embedded_hal_stable::digital::v2::OutputPin> OutputWrapper<T> {
    pub fn new(pin: T) -> OutputWrapper<T>{
        OutputWrapper{pin}
    }
}
impl<T: embedded_hal_stable::digital::v2::OutputPin> embedded_hal::digital::OutputPin for OutputWrapper<T> {
    type Error = ();

    fn try_set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low().or(Err(()))
    }

    fn try_set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high().or(Err(()))
    }
}

pub struct TimerWrapper<T>{
    pub timer: T
}
impl<T: embedded_hal_stable::timer::CountDown> TimerWrapper<T> {
    pub fn new(timer: T) -> TimerWrapper<T>{
        TimerWrapper{timer}
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

impl<T: embedded_hal_stable::timer::CountDown> embedded_hal::timer::CountDown for TimerWrapper<T>
    where T::Time: From<u64>
{
    type Error = Void;
    type Time = NanosecWrapper;

    fn try_start<G>(&mut self, count: G) -> Result<(), Self::Error> where G: Into<Self::Time>  {
        let ns = count.into().ns;
        Ok(self.timer.start(ns as u64))
    }

    fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
        self.timer.wait()
    }
}


static mut BOARD_RESOURCE_BUILDER_TAKEN:EspMutex<bool> = EspMutex::new(false);
impl BoardResourceBuilder{
    pub fn resolve(target_accel: MCDelay) -> Option<BoardResources> {
        esp_idf_svc::log::EspLogger::initialize_default();
        unsafe {
            BOARD_RESOURCE_BUILDER_TAKEN.lock(|taken| {
                if *taken {
                    Option::None
                } else {
                    *taken = true;

                    println!("taking peripherals");
                    let pr = Peripherals::take().unwrap();
                    let gpio = pr.GPIO.split();
                    let stp1_step = OutputWrapper::new(gpio.gpio2.into_push_pull_output());
                    let stp1_dir = OutputWrapper::new(gpio.gpio5.into_push_pull_output());
                    println!("allocating new pins");
                    let stp1_reset = OutputWrapper::new(gpio.gpio13.into_push_pull_output());
                    let stp1_m0 = OutputWrapper::new(gpio.gpio16.into_push_pull_output());
                    let stp1_m1 = OutputWrapper::new(gpio.gpio17.into_push_pull_output());
                    let stp1_m2 = OutputWrapper::new(gpio.gpio18.into_push_pull_output());
                    let stp2_step = OutputWrapper::new(gpio.gpio4.into_push_pull_output());
                    let stp2_dir = OutputWrapper::new(gpio.gpio15.into_push_pull_output());
                    let stp1_lim = gpio.gpio21.into_pull_down_input();
                    let stp2_lim = gpio.gpio22.into_pull_down_input();
                    println!("Pins allocated");
                    let (_, dport_clock_control) = pr.DPORT.split();

                    let clkcntrl = esp32_hal::clock_control::ClockControl::new(
                        pr.RTCCNTL,
                        pr.APB_CTRL,
                        dport_clock_control,
                        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
                    )
                        .unwrap();
                    println!("abp freq:{}, rtc_nanosecs:{}, xtal freq:{}",
                             clkcntrl.apb_frequency(),
                             clkcntrl.rtc_nanoseconds(),
                             clkcntrl.xtal_frequency_from_scratch().unwrap()
                    );


                    let (clkcntrl_config, mut watchdog_rtc) = clkcntrl.freeze().unwrap();
                    let (mut timer0, mut timer1, mut timer2, mut watchdog0) = Timer::new(pr.TIMG0, clkcntrl_config);
                    watchdog0.disable();
                    watchdog_rtc.disable();
                    timer0.enable(true);
                    timer1.enable(true);
                    let mut wrapped_timer_0 = TimerWrapper::new(timer0);
                    let mut wrapped_timer_1 = TimerWrapper::new(timer1);
                    let profile_1 = ramp_maker::Trapezoidal::new(target_accel);
                    let profile_2 = ramp_maker::Trapezoidal::new(target_accel);
                    let step_driver_1 = Stepper::from_driver(DRV8825::new())
                        .enable_direction_control(stp1_dir, Direction::Forward, wrapped_timer_0.borrow_mut())
                        .unwrap()
                        .enable_step_control(stp1_step)
                        .enable_step_mode_control((stp1_reset, stp1_m0, stp1_m1, stp1_m2), stepper::step_mode::StepMode32::M32, wrapped_timer_0.borrow_mut()).unwrap()
                        .enable_motion_control((wrapped_timer_0, profile_1, DelayToTicks));

                    let step_driver_2 = Stepper::from_driver(DRV8825::new())
                        .enable_direction_control(stp2_dir, Direction::Forward, &mut wrapped_timer_1)
                        .unwrap()
                        .enable_step_control(stp2_step)
                        .enable_motion_control((wrapped_timer_1, profile_2, DelayToTicks));
                    let mut wifi1 = EspWifi::new(Arc::new(EspNetifStack::new().unwrap()), Arc::new(EspSysLoopStack::new().unwrap()), Arc::new(EspDefaultNvs::new().unwrap())).unwrap();
                    Some(BoardResources {
                        steppers: [Some(StepperContainer::stepper1(step_driver_1.release())),
                        Some(StepperContainer::stepper2(step_driver_2.release()))],
                        switches: [Some(SwitchContainer::Gpio17InPD(stp1_lim)),Some(SwitchContainer::Gpio18InPD(stp2_lim))],
                        wifis: [Some(WifiContainer::Wifi0(wifi1))],
                        servers: [Some(ServerContainer::Server0(EspWebServer::new()))],
                        clients: [Some(ClientContainer{})]
                    })
                }
            })
        }
    }
}

impl HasStepper for BoardResources
{
    type Resource = StepperContainer;
    fn take_stepper(&mut self, id: usize) -> Option<Self::Resource> {
        self.steppers.get_mut(id)?.take()
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
