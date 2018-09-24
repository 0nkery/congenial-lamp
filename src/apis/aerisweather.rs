use std::env;

use chrono::{TimeZone, Utc};
use reqwest::{Url, UrlError};

use super::{WeatherAPI, WeatherData, WeatherDataVec};

pub struct AerisWeather {
    client_id: String,
    client_secret: String,
}

impl AerisWeather {
    pub fn new() -> Result<Self, env::VarError> {
        let client_id = env::var("AERISWEATHER_CLIENT_ID")?;
        let client_secret = env::var("AERISWEATHER_CLIENT_SECRET")?;

        Ok(Self {
            client_id,
            client_secret,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AerisWeatherPeriod {
    timestamp: i64,
    avg_temp_c: f32,
}

#[derive(Deserialize)]
struct AerisWeatherForecast {
    periods: [AerisWeatherPeriod; 5],
}

#[derive(Deserialize)]
pub struct AerisWeatherResponse {
    success: bool,
    response: AerisWeatherForecast,
}

impl Into<Option<WeatherDataVec>> for AerisWeatherResponse {
    fn into(self) -> Option<WeatherDataVec> {
        if self.success {
            Some(
                self.response
                    .periods
                    .iter()
                    .map(|forecast| WeatherData {
                        date: Utc.timestamp(forecast.timestamp, 0).date(),
                        temperature: forecast.avg_temp_c,
                    }).collect::<WeatherDataVec>(),
            )
        } else {
            None
        }
    }
}

impl WeatherAPI for AerisWeather {
    type Response = AerisWeatherResponse;

    fn weekly_request_url(&self, city: &str, country: &str) -> Result<Url, UrlError> {
        Url::parse_with_params(
            &format!("https://api.aerisapi.com/forecasts/{},{}", city, country),
            &[
                ("limit", "5"),
                ("filter", "precise"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ],
        )
    }
}
