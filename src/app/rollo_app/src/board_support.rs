use std::convert::Infallible;
use std::fmt::Error;
use std::ops::{Mul, Sub};
use std::sync::Arc;
use mutex_trait::prelude::*;
#[cfg(not(feature = "ulp"))]
use esp_idf_sys::EspMutex;
use stepper::drivers::drv8825;
use stepper::drivers::drv8825::DRV8825;
use stepper::traits::{EnableDirectionControl, EnableStepControl};
use embedded_hal_stable::digital::v2::OutputPin;
use embedded_hal_stable::timer::CountDown;
use embedded_time::duration::Microseconds;
use embedded_time::rate::Microhertz;
use embedded_time::timer::param::None;
use esp32_hal::clock_control::{ClockControl, ClockControlConfig};
use esp32_hal::prelude::*;
use esp32_hal::gpio::*;
use esp32_hal::timer::*;
use esp32_hal::clock_control::*;
use esp32_hal::dport::Split;
use esp32_hal::target::{Peripherals, TIMG0, TIMG1};
use esp32_hal::units;
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_svc::wifi::EspWifi;
use stepper::{Direction, motion_control, ramp_maker, SignalError, Stepper};
use stepper::embedded_time::duration::Nanoseconds;
use stepper::motion_control::SoftwareMotionControl;
use stepper::ramp_maker::Trapezoidal;
use nb;
use void::Void;
use crate::board_support::InputPinResource::*;
use crate::board_support::TimerResource::*;
use crate::board_support::StepperResource::*;
use fixed::types::extra::U64;
use num_traits::real::Real;
use num_traits::ToPrimitive;
use crate::board_support::WifiResource::Wifi0;
use smallbox::SmallBox;



macro_rules! polymorphic_enum {
    ($name:ident $macro1:ident $macro2:ident, $($variant:ident($type:path),)*) => {
        pub enum $name { $($variant($type)),* }
        macro_rules! $macro1 {
            ($on:expr, |$with:ident| $body:block) => {
                match $on {
                    $($name::$variant($with) => $body )*
                }
            }
        }
        macro_rules! $macro2 {
            ($on:expr) => {
                match $on {
                    $($name::$variant(t) => {t} )*
                }
            }
        }

    }
}

polymorphic_enum! {
    WifiResource use_wifi take_wifi,
    Wifi0(EspWifi),
}
polymorphic_enum! {
    InputPinResource use_input_pin take_input_pin,
    Gpio17InPD(Gpio17<Input<PullDown>>),
    Gpio18InPD(Gpio18<Input<PullDown>>),
}

polymorphic_enum! {
    TimerResource use_timer take_timer,
    TimerG00(TimerWrapper<Timer<TIMG0, Timer0>>),
    TimerG01(TimerWrapper<Timer<TIMG0, Timer1>>),
}

polymorphic_enum! {
    StepperResource use_stepper take_stepper,
    stepper1(Stepper<SoftwareMotionControl<DRV8825<(), (), (), (), (), (), (), OutputWrapper<Gpio2<Output<PushPull>>>, OutputWrapper<Gpio5<Output<PushPull>>>>, TimerWrapper<Timer<TIMG0, Timer0>>, Trapezoidal<MCDelay>, DelayToTicks>>),
    stepper2(Stepper<SoftwareMotionControl<DRV8825<(), (), (), (), (), (), (), OutputWrapper<Gpio4<Output<PushPull>>>, OutputWrapper<Gpio15<Output<PushPull>>>>, TimerWrapper<Timer<TIMG0, Timer1>>, Trapezoidal<MCDelay>, DelayToTicks>>),
}
pub type MCDelay = fixed::FixedI128<U64>;
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
pub struct AxisResource {
    pub stepper_res: StepperResource,
    pub limit_sw_pin: InputPinResource,

}
pub struct BoardResources {
    pub axis_a_res: AxisResource,
    pub axis_b_res: AxisResource,
    pub wifi: WifiResource,
}
pub struct BoardResourceBuilder {}

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
        println!("Delay ns:{}", ns);
        Ok(self.timer.start(ns as u64))
    }

    fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
        self.timer.wait()
    }
}
static mut BOARD_RESOURCE_BUILDER_TAKEN:EspMutex<bool> = EspMutex::new(false);
impl<'a> BoardResourceBuilder {
    pub fn resolve(target_accel: MCDelay) -> Option<BoardResources> {
        unsafe {
            BOARD_RESOURCE_BUILDER_TAKEN.lock(|taken| {
                if *taken {
                    Option::None
                } else {
                    *taken = true;
                    let pr = Peripherals::take().unwrap();
                    let gpio = pr.GPIO.split();
                    let stp1_step = OutputWrapper::new(gpio.gpio2.into_push_pull_output());
                    let stp1_dir = OutputWrapper::new(gpio.gpio5.into_push_pull_output());
                    let stp2_step = OutputWrapper::new(gpio.gpio4.into_push_pull_output());
                    let stp2_dir = OutputWrapper::new(gpio.gpio15.into_push_pull_output());
                    let stp1_lim = gpio.gpio17.into_pull_down_input();
                    let stp2_lim = gpio.gpio18.into_pull_down_input();
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
                        .enable_direction_control(stp1_dir, Direction::Forward, &mut wrapped_timer_0)
                        .unwrap()
                        .enable_step_control(stp1_step)
                        .enable_motion_control((wrapped_timer_0, profile_1, DelayToTicks));

                    let step_driver_2 = Stepper::from_driver(DRV8825::new())
                        .enable_direction_control(stp2_dir, Direction::Forward, &mut wrapped_timer_1)
                        .unwrap()
                        .enable_step_control(stp2_step)
                        .enable_motion_control((wrapped_timer_1, profile_2, DelayToTicks));
                    let wifi = EspWifi::new(Arc::new(EspNetifStack::new().unwrap()), Arc::new(EspSysLoopStack::new().unwrap()), Arc::new(EspDefaultNvs::new().unwrap())).unwrap();
                    Some(BoardResources {
                        axis_a_res: AxisResource { stepper_res: stepper1(step_driver_1), limit_sw_pin: Gpio17InPD(stp1_lim) },
                        axis_b_res: AxisResource { stepper_res: stepper2(step_driver_2), limit_sw_pin: Gpio18InPD(stp2_lim) },
                        wifi: Wifi0(wifi)
                    })
                }
            })
        }
    }
}
