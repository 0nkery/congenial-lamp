use std::env;

use chrono::{TimeZone, Utc};
use failure::Error;
use reqwest::Url;

use apis::{WeatherAPI, WeatherData, WeatherDataVec, WeatherQuery};

pub struct WeatherBit {
    key: String,
}

impl WeatherBit {
    pub fn new() -> Result<Self, Error> {
        let key = env::var("WEATHERBIT_API_KEY")?;

        Ok(Self { key })
    }
}

impl WeatherAPI for WeatherBit {
    type Response = WeatherBitResponse;
    fn make_url(&self, query: &WeatherQuery) -> Result<Url, Error> {
        Ok(Url::parse_with_params(
            "https://api.weatherbit.io/v2.0/forecast/daily",
            &[
                ("key", &self.key),
                ("city", &query.city),
                ("country", &query.country),
            ],
        )?)
    }
}

#[derive(Deserialize)]
struct WeatherBitForecast {
    ts: i64,
    temp: f32,
}

#[derive(Deserialize)]
pub struct WeatherBitResponse {
    data: [WeatherBitForecast; 16],
}

impl Into<WeatherDataVec> for WeatherBitResponse {
    fn into(self) -> WeatherDataVec {
        self.data
            .iter()
            .map(|forecast| WeatherData {
                date: Utc.timestamp(forecast.ts, 0).naive_utc().date(),
                temperature: forecast.temp,
            }).collect::<WeatherDataVec>()
    }
}
