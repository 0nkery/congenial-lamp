use std::sync::Arc;

use actix::{Actor, Context, Handler};
use failure::Error;
use futures::Future;
use reqwest::async::Client;

use apis::{WeatherAPI, WeatherDataVec, WeatherQuery};

pub struct WeatherAPIActor<A>
where
    A: WeatherAPI + 'static,
{
    client: Arc<Client>,
    api: A,
}

impl<A> WeatherAPIActor<A>
where
    A: WeatherAPI,
{
    pub fn new(client: Arc<Client>, api: A) -> Self {
        Self { client, api }
    }
}

impl<A> Actor for WeatherAPIActor<A>
where
    A: WeatherAPI,
{
    type Context = Context<Self>;
}

impl<A, R> Handler<WeatherQuery> for WeatherAPIActor<A>
where
    A: WeatherAPI<Response = R>,
    R: Into<WeatherDataVec> + 'static,
    R: for<'de> ::serde::Deserialize<'de>,
{
    type Result = Box<Future<Item = WeatherDataVec, Error = Error>>;

    fn handle(&mut self, msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
        let url = self.api.make_url(&msg).expect("Failed to prepare URL");

        let req = self
            .client
            .get(url)
            .send()
            .and_then(|mut res| res.json::<A::Response>())
            .map(|res| res.into())
            .map_err(|err| Error::from(err));

        Box::new(req)
    }
}
