mod openweathermap;

use reqwest::r#async::RequestBuilder;
use reqwest::Method;
use smallvec::SmallVec;

pub struct WeatherData {
    temperature: f32,
    date: chrono::Date<chrono::Utc>,
}

pub type WeatherDataVec = SmallVec<[WeatherData; 32]>;

pub trait WeatherAPI {
    const BASE_URL: &'static str;
    const METHOD: Method = Method::GET;

    type Response;
    type Error;

    fn build_weekly_request(
        &self,
        req_builder: RequestBuilder,
        city: &str,
        country: Option<&str>,
    ) -> RequestBuilder;

    fn parse_weekly_response(&self, response: Self::Response) -> WeatherDataVec;
}
