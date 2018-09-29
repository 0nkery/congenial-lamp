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

mod actors;
mod apis;

// fn prepare_request<A, R>(
//     client: &reqwest::async::Client,
//     city: &str,
//     country: &str,
//     api: A,
// ) -> impl Future<Item = Option<apis::WeatherDataVec>, Error = reqwest::Error>
// where
//     A: apis::WeatherAPI<Response = R>,
//     R: Into<Option<apis::WeatherDataVec>>,
//     for<'de> R: serde::Deserialize<'de>,
// {
//     let url = api.weekly_request_url(city, country).unwrap();
//     let builder = client.request(A::METHOD, url);

//     api.build_weekly_request(builder, city, country)
//         .send()
//         .and_then(|mut res| res.json::<R>())
//         .map(|res| res.into())
// }

// #[derive(Deserialize)]
// struct QueryParams {
//     country: String,
//     city: String,
// }

// fn index(
//     req: Query<QueryParams>,
// ) -> Box<Future<Item = HttpResponse, Error = actix_web::error::InternalError<&'static str>>> {
//     let aeris_api = apis::AerisWeather::new().unwrap();
//     let apixu_api = apis::Apixu::new().unwrap();
//     let openweathermap_api = apis::OpenWeatherMap::new().unwrap();
//     let weatherbit_api = apis::WeatherBit::new().unwrap();

//     let client = reqwest::async::Client::new();

//     let req1 = prepare_request(&client, &req.city, &req.country, aeris_api)
//         .map_err(|err| println!("{}", err));

//     let req2 = prepare_request(&client, &req.city, &req.country, openweathermap_api)
//         .map_err(|err| println!("{}", err));

//     let req3 = prepare_request(&client, &req.city, &req.country, apixu_api)
//         .map_err(|err| println!("{}", err));

//     let req4 = prepare_request(&client, &req.city, &req.country, weatherbit_api)
//         .map_err(|err| println!("{}", err));

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

fn main() {
    let bind_to = std::env::var("ADDRESS").unwrap_or("127.0.0.1:8088".to_string());

    env_logger::init();

    let sys = actix::System::new("forecast");

    server::new(|| {
        App::new().middleware(middleware::Logger::default())
        // .resource("/forecast", |r| r.method(http::Method::GET).with(index))
    }).bind(&bind_to)
    .expect(&format!("Failed to bind to address {}", bind_to))
    .start();

    info!("Running server on {}", bind_to);
    sys.run();
}
