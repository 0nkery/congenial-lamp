use actix::Recipient;
use actix_web::{error, http, middleware, App, FromRequest, HttpRequest, HttpResponse, Json, Path};
use chrono::{NaiveDate, ParseError};
use failure::Error;
use futures::Future;

use apis::{WeatherData, WeatherQuery};

/// Перечисление с ошибками API. `UnexpectedError` логируются
/// полностью, а наружу отдаются без подробностей.
#[derive(Fail, Debug)]
enum APIError {
    #[fail(display = "failed to parse date - {}", _0)]
    InvalidDate(ParseError),
    #[fail(display = "invalid parameters - {}", _0)]
    BadRequest(error::Error),
    #[fail(display = "weather data not found for given day - {}", _0)]
    NotFound(NaiveDate),
    #[fail(display = "insufficient weather data for full weekly forecast")]
    InsufficientData,
    #[fail(display = "unexpected error during request - {}", _0)]
    UnexpectedError(Error),
}

/// Вспомогательная структура для упаковки ошибок в JSON.
#[derive(Serialize, Deserialize)]
struct APIErrorResponse {
    error: String,
}

impl error::ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match *self {
            APIError::InvalidDate(_) | APIError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
            APIError::NotFound(_) => http::StatusCode::NOT_FOUND,
            APIError::InsufficientData => http::StatusCode::SERVICE_UNAVAILABLE,
            APIError::UnexpectedError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let mut builder = HttpResponse::build(status_code);

        match *self {
            APIError::UnexpectedError(_) => builder.json(APIErrorResponse {
                error: "An internal error occurred. Please try again later.".to_string(),
            }),
            _ => builder.json(APIErrorResponse {
                error: format!("{}", self),
            }),
        }
    }
}

type APIResponder<D> = Box<Future<Item = Result<Json<D>, APIError>, Error = APIError>>;

impl APIError {
    fn into_responder<D: 'static>(self) -> APIResponder<D> {
        Box::new(::futures::future::err(self))
    }
}

/// Состояние для Actix' App. `WebAPI` требуется актор, который
/// будет отвечать на запросы о погоде.
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

    /// Возвращает прогноз на 5 дней, даже если от агрегатора вернулось больше.
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
                    if data.iter().any(|e| e.is_none()) {
                        return Err(APIError::InsufficientData);
                    }
                    Ok(Json(data))
                }
                Err(reason) => Err(APIError::UnexpectedError(Error::from(reason))),
            }).map_err(|err| APIError::UnexpectedError(Error::from(err)));

        Box::new(data)
    }
}

#[cfg(test)]
mod test {
    use actix::prelude::*;
    use actix_web::{test, HttpMessage};
    use chrono::Utc;
    use failure::err_msg;

    use super::*;
    use apis::WeatherDataVec;

    struct TestWeatherActor;

    impl Actor for TestWeatherActor {
        type Context = SyncContext<Self>;
    }

    impl Handler<WeatherQuery> for TestWeatherActor {
        type Result = Result<WeatherDataVec, Error>;

        fn handle(&mut self, _msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
            let mut vec = WeatherDataVec::new();

            for _ in 0..5 {
                vec.push(WeatherData {
                    temperature: 10.0,
                    date: Utc::now().naive_utc().date(),
                });
            }

            Ok(vec)
        }
    }

    struct EmptyWeatherActor;

    impl Actor for EmptyWeatherActor {
        type Context = SyncContext<Self>;
    }

    impl Handler<WeatherQuery> for EmptyWeatherActor {
        type Result = Result<WeatherDataVec, Error>;

        fn handle(&mut self, _msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
            let vec = WeatherDataVec::new();
            Ok(vec)
        }
    }

    struct FailingWeatherActor;

    impl Actor for FailingWeatherActor {
        type Context = SyncContext<Self>;
    }

    impl Handler<WeatherQuery> for FailingWeatherActor {
        type Result = Result<WeatherDataVec, Error>;

        fn handle(&mut self, _msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
            Err(err_msg("test"))
        }
    }

    fn init_test_server<F: Fn() -> WebAPI + Sync + Send + 'static>(init_fn: F) -> test::TestServer {
        test::TestServer::build_with_state(init_fn).start(|app: &mut test::TestApp<WebAPI>| {
            app.resource("/forecast/daily/{country}/{city}/{day}", |r| {
                r.method(http::Method::GET).f(WebAPI::daily_forecast)
            }).resource("/forecast/weekly/{country}/{city}", |r| {
                r.method(http::Method::GET).f(WebAPI::weekly_forecast)
            });
        })
    }

    #[test]
    fn normal_path() {
        let mut srv = init_test_server(|| {
            let weather_actor = SyncArbiter::start(1, || TestWeatherActor {});
            WebAPI {
                aggregator: weather_actor.recipient(),
            }
        });

        let now = Utc::now().format("%Y-%m-%d");

        let request = srv
            .client(
                http::Method::GET,
                &format!("/forecast/daily/UK/London/{}", now),
            ).finish()
            .expect("Failed to construct test request");
        let response = srv
            .execute(request.send())
            .expect("Failed to send test request");

        assert!(response.status().is_success());

        let data: WeatherData = srv
            .execute(response.json())
            .expect("Failed to parse response as JSON");

        assert_eq!(data.temperature, 10.0);

        let request = srv
            .client(http::Method::GET, "/forecast/weekly/UK/London")
            .finish()
            .expect("Failed to construct test request");
        let response = srv
            .execute(request.send())
            .expect("Failed to send test request");

        assert!(response.status().is_success());

        let data: [WeatherData; 5] = srv
            .execute(response.json())
            .expect("Failed to parse response as JSON");
        assert_eq!(data[0].temperature, 10.0);
        assert_eq!(data[4].temperature, 10.0);
    }

    #[test]
    fn error_path() {
        let mut srv = init_test_server(|| {
            let weather_actor = SyncArbiter::start(1, || TestWeatherActor {});
            WebAPI {
                aggregator: weather_actor.recipient(),
            }
        });

        let request = srv
            .client(http::Method::GET, "/forecast/daily/UK/London/invalid-date")
            .finish()
            .expect("Failed to construct test request");
        let response = srv
            .execute(request.send())
            .expect("Failed to send test request");

        assert!(response.status().is_client_error());

        let data: APIErrorResponse = srv
            .execute(response.json())
            .expect("Failed to parse response as JSON");

        assert_eq!(
            data.error,
            "failed to parse date - input contains invalid characters"
        );

        let request = srv
            .client(http::Method::GET, "/forecast/daily/UK/London/2077-01-01")
            .finish()
            .expect("Failed to construct test request");
        let response = srv
            .execute(request.send())
            .expect("Failed to send test request");

        assert!(response.status().is_client_error());

        let data: APIErrorResponse = srv
            .execute(response.json())
            .expect("Failed to parse response as JSON");

        assert_eq!(
            data.error,
            "weather data not found for given day - 2077-01-01"
        );
    }

    #[test]
    fn empty_data() {
        let mut srv = init_test_server(|| {
            let weather_actor = SyncArbiter::start(1, || EmptyWeatherActor {});
            WebAPI {
                aggregator: weather_actor.recipient(),
            }
        });

        let request = srv
            .client(http::Method::GET, "/forecast/weekly/UK/London")
            .finish()
            .expect("Failed to construct test request");
        let response = srv
            .execute(request.send())
            .expect("Failed to send test request");

        assert!(response.status().is_server_error());

        let data: APIErrorResponse = srv
            .execute(response.json())
            .expect("Failed to parse response as JSON");
        assert_eq!(
            data.error,
            "insufficient weather data for full weekly forecast"
        )
    }

    #[test]
    fn failing_actor() {
        let mut srv = init_test_server(|| {
            let weather_actor = SyncArbiter::start(1, || FailingWeatherActor {});
            WebAPI {
                aggregator: weather_actor.recipient(),
            }
        });

        let request = srv
            .client(http::Method::GET, "/forecast/weekly/UK/London")
            .finish()
            .expect("Failed to construct test request");
        let response = srv
            .execute(request.send())
            .expect("Failed to send test request");

        assert!(response.status().is_server_error());

        let data: APIErrorResponse = srv
            .execute(response.json())
            .expect("Failed to parse response as JSON");
        assert_eq!(
            data.error,
            "An internal error occurred. Please try again later."
        )
    }
}
