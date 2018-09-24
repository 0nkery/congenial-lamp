use std::env;

use chrono::{TimeZone, Utc};
use reqwest::{Url, UrlError};

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
    type Response = WeatherBitResponse;

    fn weekly_request_url(&self, city: &str, country: &str) -> Result<Url, UrlError> {
        Url::parse_with_params(
            "https://api.weatherbit.io/v2.0/forecast/daily",
            &[("city", city), ("country", country), ("key", &self.key)],
        )
    }
}
