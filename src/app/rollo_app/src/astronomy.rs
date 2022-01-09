use time::{Time, Duration};
use crate::ClientContainer;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not execute request: {0}")]
    Client(bal::client::Error),
    #[error("URL Parsing error")]
    UrlParsing,
    #[error("Response Parsing error")]
    ResponseParsing,
    #[error("Time Parsing error")]
    TimeParsing,
    #[error("What?")]
    Undefined,
}

#[derive(Debug)]
pub  struct SunState {
    pub current_time: Time,
    pub sunset: Time,
    pub sunrise: Time,
}
#[derive(PartialEq, Debug)]
pub enum SunPosition{
    Up,
    Down,
}

impl SunState {
    pub fn get_current_sun_pos(&self) -> SunPosition {
        // todo this could be optimized and needs testing
        if self.current_time < self.sunrise && self.current_time > self.sunset {
            SunPosition::Down

        } else if self.current_time > self.sunrise && self.current_time < self.sunset{
            SunPosition::Up
        } else {
            if self.sunset < self.sunrise{
                SunPosition::Up
            }else {
                SunPosition::Down
            }
        }
    }

    pub fn get_time_to_next_event(&self) -> Duration {
        let mut diff = match self.get_current_sun_pos(){
            SunPosition::Up => {(self.sunset - self.current_time)}
            SunPosition::Down => {(self.sunrise - self.current_time)}
        };
        if diff.is_negative() {
            Duration::days(1) + diff
        } else {
           diff
        }
    }
}

pub trait SunStateGetter{
    fn get_sun_state(&self, client: &mut ClientContainer) -> Result<SunState, Error>;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use time::macros::time;
    #[test]
    fn test_sunstate() {
        let mut state = SunState{
            current_time: time!(02:00),
            sunset: time!(15:00),
            sunrise: time!(05:00),
        };
        assert_eq!(state.get_time_to_next_event(), Duration::hours(3));
        let res = state.get_current_sun_pos();
        assert_eq!(res, SunPosition::Down);
        state.current_time = time!(10:00);
        assert_eq!(state.get_time_to_next_event(), Duration::hours(5));
        assert_eq!(state.get_current_sun_pos(), SunPosition::Up);
        state.current_time = time!(20:00);
        assert_eq!(state.get_time_to_next_event(), Duration::hours(9));
        assert_eq!(state.get_current_sun_pos(), SunPosition::Down);

        let mut state = SunState{
            current_time: time!(02:00),
            sunset: time!(05:00),
            sunrise: time!(15:00),
        };
        assert_eq!(state.get_time_to_next_event(), Duration::hours(3));
        assert_eq!(state.get_current_sun_pos(), SunPosition::Up);
        state.current_time = time!(10:00);
        assert_eq!(state.get_time_to_next_event(), Duration::hours(5));
        assert_eq!(state.get_current_sun_pos(), SunPosition::Down);
        state.current_time = time!(20:00);
        assert_eq!(state.get_time_to_next_event(), Duration::hours(9));
        assert_eq!(state.get_current_sun_pos(), SunPosition::Up);
    }



}