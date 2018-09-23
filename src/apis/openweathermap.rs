use chrono::{TimeZone, Utc};
use crate::apis::{WeatherAPI, WeatherData, WeatherDataVec};
use itertools::Itertools;
use reqwest::r#async::RequestBuilder;
use serde_derive::Deserialize;

struct OpenWeatherMap {
    app_id: String,
}

impl OpenWeatherMap {
    pub fn new() -> Result<Self, std::env::VarError> {
        let app_id = std::env::var("OPENWEATHERMAP_API_KEY")?;

        Ok(OpenWeatherMap { app_id })
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

impl Into<WeatherDataVec> for OWMResponse {
    fn into(self) -> WeatherDataVec {
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
            }).collect::<WeatherDataVec>()
    }
}

impl WeatherAPI for OpenWeatherMap {
    const BASE_URL: &'static str = "https://api.openweathermap.org/data/2.5/forecast";

    type Error = String;
    type Response = OWMResponse;

    fn build_weekly_request(
        &self,
        req_builder: RequestBuilder,
        city: &str,
        _country: Option<&str>,
    ) -> RequestBuilder {
        req_builder.query(&[("q", city), ("units", "metric"), ("APPID", &self.app_id)])
    }
}
