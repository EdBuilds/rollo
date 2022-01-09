use serde::Deserialize;
use time::Time;
use url::Url;
use bal::networking_types::{Method, Request, Status};
use bal::client::ClientResource;
use board_support::ClientContainer;
use crate::astronomy::{Error, SunState, SunStateGetter};

#[derive(Debug, Deserialize)]
struct IpGeolocationLocation {
    ip: String,
    country_code2: String,
    country_code3: String,
    country_name: String,
    state_prov: String,
    district: String,
    city: String,
    zipcode: String,
    latitude: f32,
    longitude: f32
}
#[derive(Debug, Deserialize)]
struct IpGeolocationRequestBody {
    location: IpGeolocationLocation,
    date: String,
    current_time: String,
    sunrise: String,
    sunset: String,
    sun_status: String,
    solar_noon: String,
    day_length: String,
    sun_altitude: f32,
    sun_distance: f32,
    sun_azimuth: f32,
    moonrise: String,
    moonset: String,
    moon_status: String,
    moon_altitude: f32,
    moon_distance: f32,
    moon_azimuth: f32,
    moon_parallactic_angle: f32
}
pub struct Getter {}
impl SunStateGetter for Getter {
    fn get_sun_state(&self, client: &mut ClientContainer) -> Result<SunState, Error> {
        let mut url = Url::parse("https://api.ipgeolocation.io").map_err(|_| Error::UrlParsing)?;
        url.set_path("astronomy");
        url.set_query(Some(format!("apiKey={}", "75bdd8f840044c509ab39667550643fe").as_str()));
        let resp = client.make_request(Request{
            method: Method::Get,
            url: url.to_string(),
            body: "".to_string()
        }).map_err(|e|Error::Client(e))?;
        match resp.status {
            Status::Ok => {
                let parsed_resp: IpGeolocationRequestBody = serde_json::from_str(resp.body.as_str()).map_err(|_|Error::ResponseParsing)?;
                println!("parsed response:{:#?}", parsed_resp);
                // todo there must be a way to have this as a single parsing descriptor, but oh well..
                let hm_format = time::macros::format_description!("[hour]:[minute]");
                let hmss_format = time::macros::format_description!("[hour]:[minute]:[second].[subsecond digits:3]");
                Ok(SunState{
                    // todo Fix this mess, im going to sleep
                    current_time: Time::parse(parsed_resp.current_time.as_str(), &hmss_format).map_err(|_|Error::ResponseParsing)?,
                    sunset: Time::parse(parsed_resp.sunset.as_str(), &hm_format).map_err(|_|Error::ResponseParsing)?,
                    sunrise: Time::parse(parsed_resp.sunrise.as_str(), &hm_format).map_err(|_|Error::ResponseParsing)?,
                })
            }
            _ => {Err(Error::Undefined)}
        }
    }
}
fn get_time_from_sring(s: String) -> Result <Time, Error> {
    Ok(Time::MIDNIGHT)
}