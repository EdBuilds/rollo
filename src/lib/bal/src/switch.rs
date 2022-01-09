use std::convert::Infallible;
use embedded_hal_stable::digital::v2::InputPin;

pub trait SwitchResource{
    fn is_low(&mut self) -> Result<bool, Infallible>;
    fn is_high(&mut self) -> Result<bool, Infallible>;
}
