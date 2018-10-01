use std::env;

use chrono::{TimeZone, Utc};
use failure::Error;
use itertools::Itertools;
use reqwest::Url;

use apis::{WeatherAPI, WeatherData, WeatherDataVec, WeatherQuery};

/// https://openweathermap.org/forecast5
pub struct OpenWeatherMap {
    app_id: String,
}

impl OpenWeatherMap {
    pub fn new() -> Result<Self, Error> {
        let app_id = env::var("OPENWEATHERMAP_API_KEY")?;

        Ok(Self { app_id })
    }
}

impl WeatherAPI for OpenWeatherMap {
    type Response = OWMResponse;

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, Error> {
        Ok(Url::parse_with_params(
            "https://api.openweathermap.org/data/2.5/forecast",
            &[
                ("units", "metric"),
                ("q", &query.city),
                ("APPID", &self.app_id),
            ],
        )?)
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

#[cfg(test)]
mod test {

    use chrono::{Datelike, Duration, Utc};

    use super::*;

    #[test]
    fn aggregates_responses() {
        let mut now = Utc::now();
        let mut entries = Vec::new();

        for _ in 0..40 {
            now = now + Duration::hours(3);
            entries.push(OWMDataEntry {
                dt: now.timestamp(),
                main: OWMMainSection {
                    temp: now.day() as f32,
                },
            });
        }

        let owm_response = OWMResponse { list: entries };

        let weather_data_vec: WeatherDataVec = owm_response.into();

        assert!(weather_data_vec.len() == 5 || weather_data_vec.len() == 6);

        for entry in weather_data_vec {
            assert_eq!(entry.temperature, entry.date.day() as f32);
        }
    }
}
