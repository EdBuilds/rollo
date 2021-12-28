use std::convert::Infallible;
use embedded_hal_stable::digital::v2::InputPin;

pub trait SwitchResource = InputPin<Error = Infallible>;
