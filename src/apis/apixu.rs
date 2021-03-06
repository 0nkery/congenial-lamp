use std::env;

use chrono::{TimeZone, Utc};
use failure::Error;
use reqwest::Url;

use apis::{WeatherAPI, WeatherData, WeatherDataVec, WeatherQuery};

/// https://www.apixu.com/doc/forecast.aspx
pub struct Apixu {
    key: String,
}

const MAX_DAYS: &str = "7";

impl Apixu {
    pub fn new() -> Result<Self, Error> {
        let key = env::var("APIXU_API_KEY")?;

        Ok(Self { key })
    }
}

impl WeatherAPI for Apixu {
    type Response = ApixuResponse;

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, Error> {
        Ok(Url::parse_with_params(
            "https://api.apixu.com/v1/forecast.json",
            &[("days", MAX_DAYS), ("q", &query.city), ("key", &self.key)],
        )?)
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
                date: Utc.timestamp(forecast.date_epoch, 0).naive_utc().date(),
                temperature: forecast.day.avgtemp_c,
            }).collect::<WeatherDataVec>()
    }
}

#[cfg(test)]
mod test {
    use chrono::Duration;
    use serde_json;

    use super::*;

    #[test]
    fn parses_from_value() {
        let now = Utc::now();

        let test_json = json!({
            "forecast": {
                "forecastday": [
                    {
                        "date_epoch": now.timestamp(),
                        "day": {
                            "avgtemp_c": 10.0
                        }
                    },
                    {
                        "date_epoch": (now + Duration::days(1)).timestamp(),
                        "day": {
                            "avgtemp_c": 11.0
                        }
                    },
                    {
                        "date_epoch": (now + Duration::days(2)).timestamp(),
                        "day": {
                            "avgtemp_c": 12.0
                        }
                    },
                    {
                        "date_epoch": (now + Duration::days(3)).timestamp(),
                        "day": {
                            "avgtemp_c": 13.0
                        }
                    },
                    {
                        "date_epoch": (now + Duration::days(4)).timestamp(),
                        "day": {
                            "avgtemp_c": 14.0
                        }
                    },
                    {
                        "date_epoch": (now + Duration::days(5)).timestamp(),
                        "day": {
                            "avgtemp_c": 15.0
                        }
                    },
                    {
                        "date_epoch": (now + Duration::days(6)).timestamp(),
                        "day": {
                            "avgtemp_c": 10.0
                        }
                    }
                ]
            }
        });

        let response: ApixuResponse =
            serde_json::from_value(test_json).expect("Failed to parse test JSON");

        assert_eq!(response.forecast.forecastday[0].day.avgtemp_c, 10.0);
        assert_eq!(response.forecast.forecastday[6].day.avgtemp_c, 10.0);
    }
}
