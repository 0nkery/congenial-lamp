mod aerisweather;
mod apixu;
mod openweathermap;
mod weatherbit;

pub use self::aerisweather::AerisWeather;
pub use self::apixu::Apixu;
pub use self::openweathermap::OpenWeatherMap;
pub use self::weatherbit::WeatherBit;

use chrono::{DateTime, Utc};
use reqwest::async::RequestBuilder;
use reqwest::{Method, Url, UrlError};
use smallvec::SmallVec;

#[derive(Debug, Serialize)]
pub struct WeatherData {
    pub temperature: f32,
    // `DateTime`, потому что `chrono` не умеет serde для `Date`.
    pub date: DateTime<Utc>,
}

pub type WeatherDataVec = SmallVec<[WeatherData; 32]>;

pub trait WeatherAPI {
    const METHOD: Method = Method::GET;

    type Response: Into<Option<WeatherDataVec>>;

    fn weekly_request_url(&self, city: &str, country: &str) -> Result<Url, UrlError>;

    fn build_weekly_request(
        &self,
        req_builder: RequestBuilder,
        _city: &str,
        _country: &str,
    ) -> RequestBuilder {
        req_builder
    }
}
