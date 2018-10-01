extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate reqwest;
extern crate tokio;

extern crate chrono;
extern crate itertools;
extern crate smallvec;

extern crate env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use actix::{Arbiter, Recipient};
use actix_web::server;

mod aggregator;
mod apis;
mod weather_api;
mod web_api;

use apis::WeatherQuery;
use weather_api::WeatherAPIActor;

fn init_aggregator() -> Recipient<WeatherQuery> {
    let client = std::sync::Arc::new(reqwest::async::Client::new());

    let aerisweather = {
        let api = apis::AerisWeather::new().expect("Failed to init AerisWeather API");
        let client = client.clone();
        Arbiter::start(move |_ctx| WeatherAPIActor::new(client, api))
    };

    let apixu = {
        let api = apis::Apixu::new().expect("Failed to init Apixu API");
        let client = client.clone();
        Arbiter::start(move |_ctx| WeatherAPIActor::new(client, api))
    };

    let openweathermap = {
        let api = apis::OpenWeatherMap::new().expect("Failed to init OpenWeatherMap API");
        let client = client.clone();
        Arbiter::start(move |_ctx| WeatherAPIActor::new(client, api))
    };

    let weatherbit = {
        let api = apis::WeatherBit::new().expect("Failed to init WeatherBit");
        let client = client.clone();
        Arbiter::start(move |_ctx| WeatherAPIActor::new(client, api))
    };

    let aggregator = aggregator::Aggregator::new()
        .add_api(aerisweather.recipient())
        .add_api(apixu.recipient())
        .add_api(openweathermap.recipient())
        .add_api(weatherbit.recipient());

    Arbiter::start(|_ctx| aggregator).recipient()
}

fn main() {
    let bind_to = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8088".to_string());

    env_logger::init();

    let sys = actix::System::new("forecast");

    server::new(|| {
        let aggregator = init_aggregator();
        web_api::WebAPI::new(aggregator)
    }).bind(&bind_to)
    .unwrap_or_else(|err| panic!("Failed to bind to address {} due to {}", bind_to, err))
    .start();

    info!("Running server on {}", bind_to);
    sys.run();
}
