use std::env;
use std::rc::Rc;

use actix::{Actor, Context, Handler};
use chrono::{TimeZone, Utc};
use futures::Future;
use reqwest::{async::Client, Url, UrlError};

use apis::{WeatherData, WeatherDataVec, WeatherQuery};

pub struct Apixu {
    key: String,
    client: Rc<Client>,
}

const MAX_DAYS: &str = "7";

impl Apixu {
    pub fn new(client: Rc<Client>) -> Result<Self, env::VarError> {
        let key = env::var("APIXU_API_KEY")?;

        Ok(Self { key, client })
    }

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, UrlError> {
        Url::parse_with_params(
            "https://api.apixu.com/v1/forecast.json",
            &[("days", MAX_DAYS), ("q", &query.city), ("key", &self.key)],
        )
    }
}

impl Actor for Apixu {
    type Context = Context<Self>;
}

impl Handler<WeatherQuery> for Apixu {
    type Result = Box<Future<Item = WeatherDataVec, Error = ()>>;

    fn handle(&mut self, msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
        let url = self.make_url(&msg).expect("Failed to prepare URL");

        let req = self
            .client
            .get(url)
            .send()
            .and_then(|mut res| res.json::<ApixuResponse>())
            .map(|res| res.into())
            .map_err(|_| ());

        Box::new(req)
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
                date: Utc.timestamp(forecast.date_epoch, 0),
                temperature: forecast.day.avgtemp_c,
            }).collect::<WeatherDataVec>()
    }
}
