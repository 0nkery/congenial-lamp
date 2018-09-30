mod aerisweather;
mod apixu;
mod openweathermap;
mod weatherbit;

pub use self::aerisweather::AerisWeather;
pub use self::apixu::Apixu;
pub use self::openweathermap::OpenWeatherMap;
pub use self::weatherbit::WeatherBit;

use actix::Message;
use chrono::{DateTime, Utc};
use reqwest::{Method, Url, UrlError};
use smallvec::SmallVec;

#[derive(Debug, Serialize)]
pub struct WeatherData {
    pub temperature: f32,
    // `DateTime`, потому что `chrono` не умеет serde для `Date`.
    pub date: DateTime<Utc>,
}

pub type WeatherDataVec = SmallVec<[WeatherData; 32]>;

#[derive(Clone, Serialize, Deserialize)]
pub struct WeatherQuery {
    country: String,
    city: String,
}

unsafe impl Sync for WeatherQuery {}

impl Message for WeatherQuery {
    // TODO: more descriptive Error
    type Result = Result<WeatherDataVec, ()>;
}

pub trait WeatherAPI {
    const METHOD: Method = Method::GET;
    type Response: Into<WeatherDataVec>;

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, UrlError>;
}
