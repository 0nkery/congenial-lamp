extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate reqwest;
extern crate tokio;

extern crate chrono;
extern crate itertools;
#[macro_use]
extern crate smallvec;

extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;

use actix::{Actor, Addr};
use actix_web::server;
use failure::Error;

mod aggregator;
mod apis;
mod weather_api;
mod web_api;

use aggregator::Aggregator;
use weather_api::WeatherAPIActor;

fn init_aggregator() -> Result<Addr<Aggregator>, Error> {
    let client = std::sync::Arc::new(reqwest::async::Client::new());

    let aerisweather = {
        let api = apis::AerisWeather::new()?;
        let client = client.clone();
        WeatherAPIActor::new(client, api).start()
    };

    let apixu = {
        let api = apis::Apixu::new()?;
        let client = client.clone();
        WeatherAPIActor::new(client, api).start()
    };

    let openweathermap = {
        let api = apis::OpenWeatherMap::new()?;
        let client = client.clone();
        WeatherAPIActor::new(client, api).start()
    };

    let weatherbit = {
        let api = apis::WeatherBit::new()?;
        let client = client.clone();
        WeatherAPIActor::new(client, api).start()
    };

    let aggregator = aggregator::Aggregator::new()
        .add_api(aerisweather.recipient())
        .add_api(apixu.recipient())
        .add_api(openweathermap.recipient())
        .add_api(weatherbit.recipient());

    Ok(aggregator.start())
}

fn main() -> Result<(), Error> {
    let bind_to = std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8088".to_string());

    env_logger::init();

    let sys = actix::System::new("forecast");

    let aggregator = init_aggregator()?;

    server::new(move || {
        let addr = aggregator.clone().recipient();
        web_api::WebAPI::new(addr)
    }).bind(&bind_to)?
    .start();

    info!("Running server on {}", bind_to);
    sys.run();

    Ok(())
}
