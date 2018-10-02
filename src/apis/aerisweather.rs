use std::env;

use chrono::{TimeZone, Utc};
use failure::Error;
use reqwest::Url;

use apis::{WeatherAPI, WeatherData, WeatherDataVec, WeatherQuery};

/// https://www.aerisweather.com/support/docs/api/reference/endpoints/forecasts/
pub struct AerisWeather {
    client_id: String,
    client_secret: String,
}

impl AerisWeather {
    pub fn new() -> Result<Self, Error> {
        let client_id = env::var("AERISWEATHER_CLIENT_ID")?;
        let client_secret = env::var("AERISWEATHER_CLIENT_SECRET")?;

        Ok(Self {
            client_id,
            client_secret,
        })
    }
}

impl WeatherAPI for AerisWeather {
    type Response = AerisWeatherResponse;

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, Error> {
        Ok(Url::parse_with_params(
            &format!(
                "https://api.aerisapi.com/forecasts/{},{}",
                query.city, query.country
            ),
            &[
                ("limit", "5"),
                ("filter", "precise"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ],
        )?)
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
    response: [AerisWeatherForecast; 1],
}

impl Into<WeatherDataVec> for AerisWeatherResponse {
    fn into(self) -> WeatherDataVec {
        if self.success {
            self.response[0]
                .periods
                .iter()
                .map(|forecast| WeatherData {
                    date: Utc.timestamp(forecast.timestamp, 0).naive_utc().date(),
                    temperature: forecast.avg_temp_c,
                }).collect::<WeatherDataVec>()
        } else {
            WeatherDataVec::new()
        }
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
            "success": true,
            "response": [
                {
                    "periods": [
                        {
                            "timestamp": now.timestamp(),
                            "avgTempC": 10.0
                        },
                        {
                            "timestamp": (now + Duration::days(1)).timestamp(),
                            "avgTempC": 11.0
                        },
                        {
                            "timestamp": (now + Duration::days(2)).timestamp(),
                            "avgTempC": 12.0
                        },
                        {
                            "timestamp": (now + Duration::days(3)).timestamp(),
                            "avgTempC": 13.0
                        },
                        {
                            "timestamp": (now + Duration::days(4)).timestamp(),
                            "avgTempC": 10.0
                        }
                    ]
                }
            ]
        });

        let response: AerisWeatherResponse =
            serde_json::from_value(test_json).expect("Failed to parse test JSON");

        assert!(response.success);
        assert_eq!(response.response[0].periods[0].avg_temp_c, 10.0);
        assert_eq!(response.response[0].periods[4].avg_temp_c, 10.0);
        assert_eq!(
            response.response[0].periods[2].timestamp,
            (now + Duration::days(2)).timestamp()
        );
    }
}
