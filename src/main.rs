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

use actix_web::{http, middleware, server, App, FromRequest, HttpRequest, HttpResponse, Query};
use futures::Future;

mod aggregator;
mod apis;

#[derive(Clone)]
struct AppState {
    aggregator: actix::Recipient<apis::WeatherQuery>,
}

unsafe impl Sync for AppState {}

fn index(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = actix_web::error::InternalError<&'static str>>> {
    let query = Query::<apis::WeatherQuery>::extract(req).expect("bad request");

    let data = req
        .state()
        .aggregator
        .send(query.into_inner())
        .map(|res| match res {
            Ok(res) => {
                let body = serde_json::to_string(res.as_slice()).unwrap();
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)
            }
            _ => HttpResponse::Ok().finish(),
        }).map_err(|_| {
            actix_web::error::InternalError::new(
                "fail",
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        });

    Box::new(data)
}

fn main() -> Result<(), Box<::std::error::Error>> {
    let bind_to = std::env::var("ADDRESS").unwrap_or("127.0.0.1:8088".to_string());

    env_logger::init();

    let sys = actix::System::new("forecast");

    let client = std::sync::Arc::new(reqwest::async::Client::new());

    let aerisweather = apis::AerisWeather::new(client.clone())?;
    let aerisweather_actor = actix::Arbiter::start(|_ctx| aerisweather);

    let apixu = apis::Apixu::new(client.clone())?;
    let apixu_actor = actix::Arbiter::start(|_ctx| apixu);

    let openweathermap = apis::OpenWeatherMap::new(client.clone())?;
    let openweathermap_actor = actix::Arbiter::start(|_ctx| openweathermap);

    let weatherbit = apis::WeatherBit::new(client.clone())?;
    let weatherbit_actor = actix::Arbiter::start(|_ctx| weatherbit);

    let aggregator = aggregator::Aggregator::new()
        .add_api(aerisweather_actor.recipient())
        .add_api(apixu_actor.recipient())
        .add_api(openweathermap_actor.recipient())
        .add_api(weatherbit_actor.recipient());

    let aggregator = actix::Arbiter::start(|_ctx| aggregator).recipient();

    let state = AppState { aggregator };

    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .resource("/forecast", |r| r.method(http::Method::GET).f(index))
    }).bind(&bind_to)
    .expect(&format!("Failed to bind to address {}", bind_to))
    .start();

    info!("Running server on {}", bind_to);
    sys.run();

    Ok(())
}
