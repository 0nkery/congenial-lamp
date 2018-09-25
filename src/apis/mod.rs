pub mod aerisweather;
pub mod apixu;
pub mod openweathermap;
pub mod weatherbit;

use chrono::{Date, Utc};
use reqwest::async::RequestBuilder;
use reqwest::{Method, Url, UrlError};
use smallvec::SmallVec;

#[derive(Debug)]
pub struct WeatherData {
    temperature: f32,
    date: Date<Utc>,
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
