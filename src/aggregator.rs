use std::collections::HashMap;
use std::time;

use actix::fut::wrap_future;
use actix::prelude::*;
use chrono::{Duration, Utc};
use failure::Error;
use futures::{future, stream, Future, Stream};
use itertools::{flatten, Itertools};
use smallvec::SmallVec;

use apis::{WeatherData, WeatherDataVec, WeatherQuery};

/// Актор, агрегирующий результаты запросов в погодным API. Хранит кэш
/// таких запросов, который очищается каждый день в полночь по UTC.
pub struct Aggregator {
    weather_apis: SmallVec<[Recipient<WeatherQuery>; 32]>,
    cache: HashMap<WeatherQuery, WeatherDataVec>,
}

unsafe impl Sync for Aggregator {}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            weather_apis: SmallVec::new(),
            cache: HashMap::new(),
        }
    }

    pub fn add_api(mut self, api: Recipient<WeatherQuery>) -> Self {
        self.weather_apis.push(api);

        self
    }

    fn aggregate(mut weather_data: WeatherDataVec) -> WeatherDataVec {
        weather_data.sort_unstable_by(|entry1, entry2| entry1.date.cmp(&entry2.date));

        weather_data
            .iter()
            .group_by(|entry| entry.date)
            .into_iter()
            .map(|(day, data)| {
                let (temperature_sum, points_count) = data
                    .fold((0.0, 0.0), |(sum, count), data| {
                        (sum + data.temperature, count + 1.0)
                    });

                let avg_temperature = temperature_sum / points_count;

                WeatherData {
                    date: day,
                    temperature: avg_temperature,
                }
            }).collect::<WeatherDataVec>()
    }

    fn duration_til_next_midnight(&mut self) -> time::Duration {
        let now = Utc::now();
        let next_midnignt = (now + Duration::days(1)).date().and_hms(0, 0, 0);

        next_midnignt.signed_duration_since(now).to_std().unwrap()
    }
}

#[derive(Message)]
struct CacheCleanup;

impl Actor for Aggregator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let at_midnight = self.duration_til_next_midnight();
        ctx.notify_later(CacheCleanup, at_midnight);
    }
}

impl Handler<CacheCleanup> for Aggregator {
    type Result = ();

    fn handle(&mut self, _msg: CacheCleanup, ctx: &mut Self::Context) -> Self::Result {
        self.cache.clear();
        self.cache.shrink_to_fit();

        let at_midnight = self.duration_til_next_midnight();
        ctx.notify_later(CacheCleanup, at_midnight);
    }
}

impl Handler<WeatherQuery> for Aggregator {
    type Result = ResponseActFuture<Self, WeatherDataVec, Error>;

    fn handle(&mut self, msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
        match self.cache.get(&msg) {
            Some(entry) => {
                let entry_fut = future::ok((*entry).clone());
                Box::new(wrap_future(entry_fut))
            }
            None => {
                let requests = self.weather_apis.iter().map(|api| api.send(msg.clone()));

                let aggregated_data = stream::futures_unordered(requests)
                    .collect()
                    .map(|results| {
                        let all_data_iter = results
                            .into_iter()
                            .filter(|result| result.is_ok())
                            .map(|result| result.unwrap().into_iter());

                        let all_data = flatten(all_data_iter).collect();

                        Self::aggregate(all_data)
                    }).map_err(|err| Error::from(err));

                let msg = msg.clone();
                let update_self =
                    wrap_future::<_, Self>(aggregated_data).map(move |result, actor, _ctx| {
                        actor.cache.insert(msg, result.clone());
                        result
                    });

                Box::new(update_self)
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn aggregates_results() {
        let now = Utc::now().naive_utc().date();
        let tomorrow = now + Duration::days(2);

        let results = smallvec![
            WeatherData {
                date: now,
                temperature: 1.0,
            },
            WeatherData {
                date: now,
                temperature: 2.0,
            },
            WeatherData {
                date: tomorrow,
                temperature: 6.0,
            },
            WeatherData {
                date: tomorrow,
                temperature: 10.0,
            }
        ];

        let aggregated = Aggregator::aggregate(results);

        assert_eq!(aggregated.len(), 2);
        assert_eq!(aggregated[0].temperature, 1.5);
        assert_eq!(aggregated[1].temperature, 8.0);
    }
}
