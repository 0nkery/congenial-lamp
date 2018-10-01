use actix::Recipient;
use actix_web::{error, http, middleware, App, FromRequest, HttpRequest, HttpResponse, Path};
use chrono::NaiveDate;
use futures::Future;
use serde_json;

use apis::WeatherQuery;

pub struct WebAPI {
    aggregator: Recipient<WeatherQuery>,
}

impl WebAPI {
    pub fn new(aggregator: Recipient<WeatherQuery>) -> App<Self> {
        let state = Self { aggregator };

        App::with_state(state)
            .middleware(middleware::Logger::default())
            .resource("/forecast/daily/{country}/{city}/{day}", |r| {
                r.method(http::Method::GET).f(Self::daily_forecast)
            }).resource("/forecast/weekly/{country}/{city}", |r| {
                r.method(http::Method::GET).f(Self::weekly_forecast)
            })
    }

    fn daily_forecast(
        req: &HttpRequest<Self>,
    ) -> Box<Future<Item = HttpResponse, Error = error::InternalError<&'static str>>> {
        let (country, city, day) = Path::<(String, String, String)>::extract(req)
            .expect("bad request")
            .into_inner();

        let day = NaiveDate::parse_from_str(&day, "%Y-%m-%d").expect("Failed to parse date");

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
                error::InternalError::new("fail", http::StatusCode::INTERNAL_SERVER_ERROR)
            });

        Box::new(data)
    }

    fn weekly_forecast(req: &HttpRequest<Self>) -> HttpResponse {
        HttpResponse::Ok().finish()
    }
}
