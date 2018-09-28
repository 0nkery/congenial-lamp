use actix::{Actor, Context, Handler};
use futures::Future;

use apis::WeatherDataVec;

#[derive(Message)]
struct WeatherQuery {
    country: String,
    city: String,
}

#[derive(Default)]
struct WeatherActor;

impl Actor for WeatherActor {
    type Context = Context<Self>;
}
