extern crate actix_web;
extern crate futures;
extern crate reqwest;
extern crate tokio;

extern crate chrono;
extern crate itertools;
extern crate smallvec;

#[macro_use]
extern crate serde_derive;
extern crate serde;

use futures::Future;

mod apis;

fn prepare_request<A, R>(
    client: &reqwest::async::Client,
    city: &str,
    country: &str,
    api: A,
) -> impl Future<Item = Option<apis::WeatherDataVec>, Error = reqwest::Error>
where
    A: apis::WeatherAPI<Response = R>,
    R: Into<Option<apis::WeatherDataVec>>,
    for<'de> R: serde::Deserialize<'de>,
{
    let url = api.weekly_request_url(city, country).unwrap();
    let builder = client.request(A::METHOD, url);

    api.build_weekly_request(builder, city, country)
        .send()
        .and_then(|mut res| res.json::<R>())
        .map(|res| res.into())
}

fn main() -> Result<(), Box<std::error::Error>> {
    let aeris_api = apis::AerisWeather::new()?;
    let apixu_api = apis::Apixu::new()?;
    let openweathermap_api = apis::OpenWeatherMap::new()?;
    let weatherbit_api = apis::WeatherBit::new()?;

    let client = reqwest::async::Client::new();

    let req1 = prepare_request(&client, "Novokuznetsk", "RU", aeris_api)
        .map_err(|err| println!("{}", err));

    let req2 = prepare_request(&client, "Novokuznetsk", "RU", openweathermap_api)
        .map_err(|err| println!("{}", err));

    let req3 = prepare_request(&client, "Novokuznetsk", "RU", apixu_api)
        .map_err(|err| println!("{}", err));

    let req4 = prepare_request(&client, "Novokuznetsk", "RU", weatherbit_api)
        .map_err(|err| println!("{}", err));

    let join = req1
        .join(req2)
        .map(apis::join_two_vecs)
        .join(req3)
        .map(apis::join_two_vecs)
        .join(req4)
        .map(apis::join_two_vecs)
        .map(|vec| {
            if let Some(vec) = vec {
                let aggregate = apis::aggregate_results(vec);
                println!("{:?}", aggregate);
            }
        });

    tokio::run(join);

    Ok(())
}
