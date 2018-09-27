mod aerisweather;
mod apixu;
mod openweathermap;
mod weatherbit;

pub use self::aerisweather::AerisWeather;
pub use self::apixu::Apixu;
pub use self::openweathermap::OpenWeatherMap;
pub use self::weatherbit::WeatherBit;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use reqwest::async::RequestBuilder;
use reqwest::{Method, Url, UrlError};
use smallvec::SmallVec;

#[derive(Debug, Serialize)]
pub struct WeatherData {
    pub temperature: f32,
    // `DateTime`, потому что `chrono` не умеет serde для `Date`.
    pub date: DateTime<Utc>,
}

pub type WeatherDataVec = SmallVec<[WeatherData; 32]>;

pub trait WeatherAPI {
    const METHOD: Method = Method::GET;

    type Response: Into<Option<WeatherDataVec>>;

    fn weekly_request_url(&self, city: &str, country: &str) -> Result<Url, UrlError>;

    fn build_weekly_request(
        &self,
        req_builder: RequestBuilder,
        _city: &str,
        _country: &str,
    ) -> RequestBuilder {
        req_builder
    }
}

pub fn join_two_vecs(
    (vec1, vec2): (Option<WeatherDataVec>, Option<WeatherDataVec>),
) -> Option<WeatherDataVec> {
    match (vec1, vec2) {
        (Some(mut vec1), Some(vec2)) => {
            vec1.extend(vec2);
            Some(vec1)
        }
        (None, Some(vec)) => Some(vec),
        (Some(vec), None) => Some(vec),
        _ => None,
    }
}

pub fn aggregate_results(mut weather_data: WeatherDataVec) -> WeatherDataVec {
    weather_data.sort_unstable_by(|entry1, entry2| entry1.date.cmp(&entry2.date));

    weather_data
        .iter()
        // Нормализуем по дате (во избежание различий во времени - например, секунды отличаются).
        .group_by(|entry| entry.date.date())
        .into_iter()
        .map(|(day, data)| {
            let (temperature_sum, points_count) = data.fold((0.0, 0.0), |(sum, count), data| {
                (sum + data.temperature, count + 1.0)
            });

            let avg_temperature = temperature_sum / points_count;

            WeatherData {
                date: day.and_hms(0, 0, 0),
                temperature: avg_temperature,
            }
        }).collect::<WeatherDataVec>()
}
