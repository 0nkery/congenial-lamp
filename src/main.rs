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
use actix_web::{http, middleware, server, App, FromRequest, HttpRequest, HttpResponse, Path};
use futures::Future;

mod aggregator;
mod apis;
mod weather_api;

use apis::WeatherQuery;
use weather_api::WeatherAPIActor;

#[derive(Clone)]
struct AppState {
    aggregator: Recipient<WeatherQuery>,
}

unsafe impl Sync for AppState {}

fn daily_forecast(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = actix_web::error::InternalError<&'static str>>> {
    let (country, city, day) = Path::<(String, String, String)>::extract(req)
        .expect("bad request")
        .into_inner();

    let day = chrono::NaiveDate::parse_from_str(&day, "%Y-%m-%d").expect("Failed to parse date");

    let query = WeatherQuery::new(country, city);

    let data = req
        .state()
        .aggregator
        .send(query)
        .map(move |res| match res {
            Ok(res) => {
                let res = res.iter().find(|e| e.date == day);

                if let Some(res) = res {
                    let body = serde_json::to_string(res).unwrap();
                    HttpResponse::Ok()
                        .content_type("application/json")
                        .body(body)
                } else {
                    HttpResponse::Ok().finish()
                }
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

    let aggregator = init_aggregator();
    let state = AppState { aggregator };

    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .resource("/forecast/daily/{country}/{city}/{day}", |r| {
                r.method(http::Method::GET).f(daily_forecast)
            })
    }).bind(&bind_to)
    .unwrap_or_else(|err| panic!("Failed to bind to address {} due to {}", bind_to, err))
    .start();

    info!("Running server on {}", bind_to);
    sys.run();
}
