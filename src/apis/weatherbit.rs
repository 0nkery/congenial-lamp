use std::env;

use chrono::{TimeZone, Utc};
use reqwest::async::RequestBuilder;

use super::{WeatherAPI, WeatherData, WeatherDataVec};

struct WeatherBit {
    key: String,
}

impl WeatherBit {
    pub fn new() -> Result<Self, env::VarError> {
        let key = env::var("WEATHERBIT_API_KEY")?;

        Ok(Self { key })
    }
}

#[derive(Deserialize)]
struct WeatherBitForecast {
    ts: i64,
    temp: f32,
}

#[derive(Deserialize)]
struct WeatherBitResponse {
    data: [WeatherBitForecast; 16],
}

impl Into<WeatherDataVec> for WeatherBitResponse {
    fn into(self) -> WeatherDataVec {
        self.data
            .iter()
            .map(|forecast| WeatherData {
                date: Utc.timestamp(forecast.ts, 0).date(),
                temperature: forecast.temp,
            }).collect::<WeatherDataVec>()
    }
}

impl WeatherAPI for WeatherBit {
    const BASE_URL: &'static str = "https://api.weatherbit.io/v2.0/forecast/daily";

    type Error = String;
    type Response = WeatherBitResponse;

    fn build_weekly_request(
        &self,
        req_builder: RequestBuilder,
        city: &str,
        country: &str,
    ) -> RequestBuilder {
        req_builder.query(&[("city", city), ("country", country), ("key", &self.key)])
    }
}
