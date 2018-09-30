use std::env;

use chrono::{TimeZone, Utc};
use reqwest::{Url, UrlError};

use apis::{WeatherAPI, WeatherData, WeatherDataVec, WeatherQuery};

pub struct Apixu {
    key: String,
}

const MAX_DAYS: &str = "7";

impl Apixu {
    pub fn new() -> Result<Self, env::VarError> {
        let key = env::var("APIXU_API_KEY")?;

        Ok(Self { key })
    }
}

impl WeatherAPI for Apixu {
    type Response = ApixuResponse;

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, UrlError> {
        Url::parse_with_params(
            "https://api.apixu.com/v1/forecast.json",
            &[("days", MAX_DAYS), ("q", &query.city), ("key", &self.key)],
        )
    }
}

#[derive(Deserialize)]
struct ApixuDayStats {
    avgtemp_c: f32,
}

#[derive(Deserialize)]
struct ApixuForecastDay {
    date_epoch: i64,
    day: ApixuDayStats,
}

#[derive(Deserialize)]
struct ApixuForecast {
    forecastday: [ApixuForecastDay; 7],
}

#[derive(Deserialize)]
pub struct ApixuResponse {
    forecast: ApixuForecast,
}

impl Into<WeatherDataVec> for ApixuResponse {
    fn into(self) -> WeatherDataVec {
        self.forecast
            .forecastday
            .iter()
            .map(|forecast| WeatherData {
                date: Utc.timestamp(forecast.date_epoch, 0),
                temperature: forecast.day.avgtemp_c,
            }).collect::<WeatherDataVec>()
    }
}
