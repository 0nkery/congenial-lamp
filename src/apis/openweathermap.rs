use std::env;

use chrono::{TimeZone, Utc};
use itertools::Itertools;
use reqwest::{Url, UrlError};

use super::{WeatherAPI, WeatherData, WeatherDataVec};

struct OpenWeatherMap {
    app_id: String,
}

impl OpenWeatherMap {
    pub fn new() -> Result<Self, env::VarError> {
        let app_id = env::var("OPENWEATHERMAP_API_KEY")?;

        Ok(Self { app_id })
    }
}

#[derive(Deserialize)]
struct OWMMainSection {
    temp: f32,
}

#[derive(Deserialize)]
struct OWMDataEntry {
    dt: i64,
    main: OWMMainSection,
}

#[derive(Deserialize)]
struct OWMResponse {
    list: Vec<OWMDataEntry>,
}

impl Into<Option<WeatherDataVec>> for OWMResponse {
    fn into(self) -> Option<WeatherDataVec> {
        Some(
            self.list
                .iter()
                .group_by(|entry| Utc.timestamp(entry.dt, 0).date())
                .into_iter()
                .map(|(day, data)| {
                    let (temperature_sum, points_count) = data
                        .fold((0.0, 0.0), |(sum, count), data| {
                            (sum + data.main.temp, count + 1.0)
                        });

                    let avg_temperature = temperature_sum / points_count;

                    WeatherData {
                        date: day,
                        temperature: avg_temperature,
                    }
                }).collect::<WeatherDataVec>(),
        )
    }
}

impl WeatherAPI for OpenWeatherMap {
    type Response = OWMResponse;

    fn weekly_request_url(&self, city: &str, _country: &str) -> Result<Url, UrlError> {
        Url::parse_with_params(
            "https://api.openweathermap.org/data/2.5/forecast",
            &[("q", city), ("units", "metric"), ("APPID", &self.app_id)],
        )
    }
}
