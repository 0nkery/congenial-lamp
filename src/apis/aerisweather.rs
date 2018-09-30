use std::env;
use std::sync::Arc;

use actix::{Actor, Context, Handler};
use chrono::{TimeZone, Utc};
use futures::Future;
use reqwest::{async::Client, Url, UrlError};

use apis::{WeatherData, WeatherDataVec, WeatherQuery};

pub struct AerisWeather {
    client_id: String,
    client_secret: String,
    client: Arc<Client>,
}

impl AerisWeather {
    pub fn new(client: Arc<Client>) -> Result<Self, env::VarError> {
        let client_id = env::var("AERISWEATHER_CLIENT_ID")?;
        let client_secret = env::var("AERISWEATHER_CLIENT_SECRET")?;

        Ok(Self {
            client_id,
            client_secret,
            client,
        })
    }

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, UrlError> {
        Url::parse_with_params(
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
        )
    }
}

impl Actor for AerisWeather {
    type Context = Context<Self>;
}

impl Handler<WeatherQuery> for AerisWeather {
    type Result = Box<Future<Item = WeatherDataVec, Error = ()>>;

    fn handle(&mut self, msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
        let url = self.make_url(&msg).expect("Failed to prepare URL");

        let req = self
            .client
            .get(url)
            .send()
            .and_then(|mut res| res.json::<AerisWeatherResponse>())
            .map(|res| res.into())
            .map_err(|err| ());

        Box::new(req)
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
                    date: Utc.timestamp(forecast.timestamp, 0),
                    temperature: forecast.avg_temp_c,
                }).collect::<WeatherDataVec>()
        } else {
            WeatherDataVec::new()
        }
    }
}
