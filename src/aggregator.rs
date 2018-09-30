use actix::{Actor, Context, Handler, Recipient};
use futures::{stream, Future, Stream};
use itertools::{flatten, Itertools};

use apis::{WeatherData, WeatherDataVec, WeatherQuery};

pub struct Aggregator {
    weather_apis: ::smallvec::SmallVec<[Recipient<WeatherQuery>; 32]>,
}

impl Actor for Aggregator {
    type Context = Context<Self>;
}

impl Handler<WeatherQuery> for Aggregator {
    type Result = Box<Future<Item = WeatherDataVec, Error = ()>>;

    fn handle(&mut self, msg: WeatherQuery, _ctx: &mut Self::Context) -> Self::Result {
        // TODO: without Clone?
        let requests = self.weather_apis.iter().map(|api| api.send(msg.clone()));

        let aggregated_weather_data = stream::futures_unordered(requests)
            .collect()
            .map(|results| {
                let all_data_iter = results
                    .into_iter()
                    .filter(|result| result.is_ok())
                    .map(|result| result.unwrap().into_iter());

                let all_data = flatten(all_data_iter).collect();

                aggregate_results(all_data)
            }).map_err(|_| ());

        Box::new(aggregated_weather_data)
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
