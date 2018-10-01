use actix::Recipient;
use actix_web::{error, http, middleware, App, FromRequest, HttpRequest, HttpResponse, Json, Path};
use chrono::{NaiveDate, ParseError};
use failure::Error;
use futures::Future;

use apis::{WeatherData, WeatherQuery};

#[derive(Fail, Debug)]
enum APIError {
    #[fail(display = "failed to parse date {}", _0)]
    InvalidDate(ParseError),
    #[fail(display = "invalid parameters {}", _0)]
    BadRequest(error::Error),
    #[fail(display = "weather data not found for day {}", _0)]
    NotFound(NaiveDate),
    #[fail(display = "unexpected error during request {}", _0)]
    UnexpectedError(Error),
}

impl error::ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            APIError::InvalidDate(_) | APIError::BadRequest(_) => {
                HttpResponse::new(http::StatusCode::BAD_REQUEST)
            }
            APIError::NotFound(_) => HttpResponse::new(http::StatusCode::NOT_FOUND),
            APIError::UnexpectedError(_) => {
                HttpResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

type APIResponder<D> = Box<Future<Item = Result<Json<D>, APIError>, Error = APIError>>;

impl APIError {
    fn into_responder<D: 'static>(self) -> APIResponder<D> {
        Box::new(::futures::future::err(self))
    }
}

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

    fn daily_forecast(req: &HttpRequest<Self>) -> APIResponder<WeatherData> {
        let (country, city, day) = match Path::<(String, String, String)>::extract(req) {
            Ok(params) => params.into_inner(),
            Err(reason) => return APIError::BadRequest(reason).into_responder(),
        };

        let day = match NaiveDate::parse_from_str(&day, "%Y-%m-%d") {
            Ok(day) => day,
            Err(reason) => return APIError::InvalidDate(reason).into_responder(),
        };

        let query = WeatherQuery::new(country, city);

        let data = req
            .state()
            .aggregator
            .send(query)
            .map(move |res| match res {
                Ok(res) => res
                    .iter()
                    .find(|e| e.date == day)
                    .ok_or(APIError::NotFound(day))
                    .and_then(|res| Ok(Json(res.clone()))),
                Err(reason) => Err(APIError::UnexpectedError(Error::from(reason))),
            }).map_err(|err| APIError::UnexpectedError(Error::from(err)));

        Box::new(data)
    }

    fn weekly_forecast(req: &HttpRequest<Self>) -> APIResponder<[Option<WeatherData>; 5]> {
        let query = match Path::<WeatherQuery>::extract(req) {
            Ok(query) => query.into_inner(),
            Err(reason) => return APIError::BadRequest(reason).into_responder(),
        };

        let data = req
            .state()
            .aggregator
            .send(query)
            .map(|res| match res {
                Ok(res) => {
                    let mut data: [Option<WeatherData>; 5] = Default::default();
                    for (i, entry) in res.into_iter().take(5).enumerate() {
                        data[i] = Some(entry);
                    }
                    Ok(Json(data))
                }
                Err(reason) => Err(APIError::UnexpectedError(Error::from(reason))),
            }).map_err(|err| APIError::UnexpectedError(Error::from(err)));

        Box::new(data)
    }
}
