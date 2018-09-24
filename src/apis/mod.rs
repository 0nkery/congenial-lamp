mod apixu;
mod openweathermap;
mod weatherbit;

use chrono::{Date, Utc};
use reqwest::async::RequestBuilder;
use reqwest::Method;
use smallvec::SmallVec;

pub struct WeatherData {
    temperature: f32,
    date: Date<Utc>,
}

pub type WeatherDataVec = SmallVec<[WeatherData; 32]>;

pub trait WeatherAPI {
    const BASE_URL: &'static str;
    const METHOD: Method = Method::GET;

    type Response: Into<WeatherDataVec>;
    type Error;

    fn build_weekly_request(
        &self,
        req_builder: RequestBuilder,
        city: &str,
        country: &str,
    ) -> RequestBuilder;
}
