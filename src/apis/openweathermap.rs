use std::env;

use chrono::{TimeZone, Utc};
use itertools::Itertools;
use reqwest::{Url, UrlError};

use apis::{WeatherAPI, WeatherData, WeatherDataVec, WeatherQuery};

pub struct OpenWeatherMap {
    app_id: String,
}

impl OpenWeatherMap {
    pub fn new() -> Result<Self, env::VarError> {
        let app_id = env::var("OPENWEATHERMAP_API_KEY")?;

        Ok(Self { app_id })
    }
}

impl WeatherAPI for OpenWeatherMap {
    type Response = OWMResponse;

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, UrlError> {
        Url::parse_with_params(
            "https://api.openweathermap.org/data/2.5/forecast",
            &[
                ("units", "metric"),
                ("q", &query.city),
                ("APPID", &self.app_id),
            ],
        )
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
pub struct OWMResponse {
    list: Vec<OWMDataEntry>,
}

impl Into<WeatherDataVec> for OWMResponse {
    fn into(self) -> WeatherDataVec {
        self.list
            .iter()
            // Здесь нужно нормализовать по дате, потому что OpenWeatherMap
            // возращает по несколько записей на один день - через каждые 3 часа.
            .group_by(|entry| Utc.timestamp(entry.dt, 0).date())
            .into_iter()
            .map(|(day, data)| {
                let (temperature_sum, points_count) = data
                    .fold((0.0, 0.0), |(sum, count), data| {
                        (sum + data.main.temp, count + 1.0)
                    });

                let avg_temperature = temperature_sum / points_count;

                WeatherData {
                    date: day.naive_utc(),
                    temperature: avg_temperature,
                }
            }).collect::<WeatherDataVec>()
    }
}
