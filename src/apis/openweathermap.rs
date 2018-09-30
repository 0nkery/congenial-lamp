use std::env;
use std::sync::Arc;

use actix::{Actor, Context, Handler};
use chrono::{TimeZone, Utc};
use futures::Future;
use itertools::Itertools;
use reqwest::{async::Client, Url, UrlError};

use apis::{WeatherData, WeatherDataVec, WeatherQuery};

pub struct OpenWeatherMap {
    app_id: String,
    client: Arc<Client>,
}

impl OpenWeatherMap {
    pub fn new(client: Arc<Client>) -> Result<Self, env::VarError> {
        let app_id = env::var("OPENWEATHERMAP_API_KEY")?;

        Ok(Self { app_id, client })
    }

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

impl Actor for OpenWeatherMap {
    type Context = Context<Self>;
}

impl Handler<WeatherQuery> for OpenWeatherMap {
    type Result = Box<Future<Item = WeatherDataVec, Error = ()>>;

    fn handle(&mut self, msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
        let url = self.make_url(&msg).expect("Failed to prepare URL");

        let req = self
            .client
            .get(url)
            .send()
            .and_then(|mut res| res.json::<OWMResponse>())
            .map(|res| res.into())
            .map_err(|_| ());

        Box::new(req)
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
                    date: day.and_hms(0, 0, 0),
                    temperature: avg_temperature,
                }
            }).collect::<WeatherDataVec>()
    }
}
