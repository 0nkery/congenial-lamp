use std::env;

use chrono::{TimeZone, Utc};
use failure::Error;
use reqwest::Url;

use apis::{WeatherAPI, WeatherData, WeatherDataVec, WeatherQuery};

/// https://www.weatherbit.io/api/weather-forecast-16-day
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

#[derive(Deserialize, Serialize)]
struct WeatherBitForecast {
    ts: i64,
    temp: f32,
}

#[derive(Deserialize, Serialize)]
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

#[cfg(test)]
mod test {
    use chrono::{Datelike, Duration};
    use serde_json;

    use super::*;

    #[test]
    fn parses_from_value() {
        let now = Utc::now();
        let mut current_datetime = now;

        let mut data = Vec::new();

        for _ in 0..16 {
            data.push(WeatherBitForecast {
                ts: current_datetime.timestamp(),
                temp: current_datetime.day() as f32,
            });

            current_datetime = current_datetime + Duration::days(1);
        }

        let test_json = json!({ "data": data });

        let response: WeatherBitResponse =
            serde_json::from_value(test_json).expect("Failed to parse test JSON");

        for (i, entry) in response.data.iter().enumerate() {
            assert_eq!(entry.temp, Utc.timestamp(entry.ts, 0).day() as f32);
            assert_eq!(
                (Utc.timestamp(entry.ts, 0) - now + Duration::seconds(1)).num_days(),
                i as i64
            );
        }
    }
}
