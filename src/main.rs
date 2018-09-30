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

use actix_web::{http, middleware, server, App, HttpResponse, Query};
use futures::Future;

mod aggregator;
mod apis;

// #[derive(Deserialize)]
// struct QueryParams {
//     country: String,
//     city: String,
// }

// fn index(
//     req: Query<QueryParams>,
// ) -> Box<Future<Item = HttpResponse, Error = actix_web::error::InternalError<&'static str>>> {
//     let join = req1
//         .join(req2)
//         .map(apis::join_two_vecs)
//         .join(req3)
//         .map(apis::join_two_vecs)
//         .join(req4)
//         .map(apis::join_two_vecs)
//         .map(|vec| {
//             if let Some(vec) = vec {
//                 let aggregate = apis::aggregate_results(vec);
//                 let body = serde_json::to_string(aggregate.as_slice()).unwrap();
//                 HttpResponse::Ok()
//                     .content_type("application/json")
//                     .body(body)
//             } else {
//                 HttpResponse::Ok().finish()
//             }
//         }).map_err(|_| {
//             actix_web::error::InternalError::new(
//                 "fail",
//                 actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
//             )
//         });

//     Box::new(join)
// }

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

    actix::Arbiter::start(|_ctx| aggregator);

    server::new(|| {
        App::new().middleware(middleware::Logger::default())
        // .resource("/forecast", |r| r.method(http::Method::GET).with(index))
    }).bind(&bind_to)
    .expect(&format!("Failed to bind to address {}", bind_to))
    .start();

    info!("Running server on {}", bind_to);
    sys.run();

    Ok(())
}
