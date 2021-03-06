mod aerisweather;
mod apixu;
mod openweathermap;
mod weatherbit;

pub use self::aerisweather::AerisWeather;
pub use self::apixu::Apixu;
pub use self::openweathermap::OpenWeatherMap;
pub use self::weatherbit::WeatherBit;

use actix::Message;
use chrono::NaiveDate;
use failure::Error;
use reqwest::{Method, Url};
use smallvec::SmallVec;

/// Прогноз на определенную дату.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeatherData {
    pub temperature: f32,
    pub date: NaiveDate,
}

pub type WeatherDataVec = SmallVec<[WeatherData; 32]>;

/// Запрос всей имеющейся информации по городу в некой стране.
#[derive(Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct WeatherQuery {
    country: String,
    city: String,
}

impl WeatherQuery {
    pub fn new(country: String, city: String) -> Self {
        Self { country, city }
    }
}

impl Message for WeatherQuery {
    type Result = Result<WeatherDataVec, Error>;
}

/// Типаж, позволяющий создавать запросы к некому погодному API.
/// Запрос должен конвертироваться в `WeatherDataVec`.
pub trait WeatherAPI {
    const METHOD: Method = Method::GET;
    type Response: Into<WeatherDataVec>;

    fn make_url(&self, query: &WeatherQuery) -> Result<Url, Error>;
}
