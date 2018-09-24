use std::env;

use chrono::{TimeZone, Utc};
use reqwest::{Url, UrlError};

use super::{WeatherAPI, WeatherData, WeatherDataVec};

struct Apixu {
    key: String,
}

impl Apixu {
    pub fn new() -> Result<Self, env::VarError> {
        let key = env::var("APIXU_API_KEY")?;

        Ok(Self { key })
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
struct ApixuResponse {
    forecast: ApixuForecast,
}

impl Into<WeatherDataVec> for ApixuResponse {
    fn into(self) -> WeatherDataVec {
        self.forecast
            .forecastday
            .iter()
            .map(|forecast| WeatherData {
                date: Utc.timestamp(forecast.date_epoch, 0).date(),
                temperature: forecast.day.avgtemp_c,
            }).collect::<WeatherDataVec>()
    }
}

impl WeatherAPI for Apixu {
    type Response = ApixuResponse;

    fn weekly_request_url(&self, city: &str, _country: &str) -> Result<Url, UrlError> {
        Url::parse_with_params(
            "https://api.apixu.com/v1/forecast.json",
            &[("q", city), ("key", &self.key)],
        )
    }
}
