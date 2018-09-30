use std::env;
use std::rc::Rc;

use actix::{Actor, Context, Handler};
use chrono::{TimeZone, Utc};
use futures::Future;
use reqwest::{async::Client, Url, UrlError};

use apis::{WeatherData, WeatherDataVec, WeatherQuery};

pub struct WeatherBit {
    key: String,
    client: Rc<Client>,
}

impl WeatherBit {
    pub fn new(client: Rc<Client>) -> Result<Self, env::VarError> {
        let key = env::var("WEATHERBIT_API_KEY")?;

        Ok(Self { key, client })
    }

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, UrlError> {
        Url::parse_with_params(
            "https://api.weatherbit.io/v2.0/forecast/daily",
            &[
                ("key", &self.key),
                ("city", &query.city),
                ("country", &query.country),
            ],
        )
    }
}

impl Actor for WeatherBit {
    type Context = Context<Self>;
}

impl Handler<WeatherQuery> for WeatherBit {
    type Result = Box<Future<Item = WeatherDataVec, Error = ()>>;

    fn handle(&mut self, msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
        let url = self.make_url(&msg).expect("Failed to prepare URL");

        let req = self
            .client
            .get(url)
            .send()
            .and_then(|mut res| res.json::<WeatherBitResponse>())
            .map(|res| res.into())
            .map_err(|_| ());

        Box::new(req)
    }
}

#[derive(Deserialize)]
struct WeatherBitForecast {
    ts: i64,
    temp: f32,
}

#[derive(Deserialize)]
pub struct WeatherBitResponse {
    data: [WeatherBitForecast; 16],
}

impl Into<WeatherDataVec> for WeatherBitResponse {
    fn into(self) -> WeatherDataVec {
        self.data
            .iter()
            .map(|forecast| WeatherData {
                date: Utc.timestamp(forecast.ts, 0),
                temperature: forecast.temp,
            }).collect::<WeatherDataVec>()
    }
}
