extern crate futures;
extern crate reqwest;

extern crate chrono;
extern crate itertools;
extern crate smallvec;

#[macro_use]
extern crate serde_derive;
extern crate serde;

mod apis;

fn main() -> Result<(), Box<std::error::Error>> {
    let aeris_api = apis::aerisweather::AerisWeather::new()?;
    let apixu_api = apis::apixu::Apixu::new()?;
    let openweathermap_api = apis::openweathermap::OpenWeatherMap::new()?;
    let weatherbit_api = apis::weatherbit::WeatherBit::new()?;

    Ok(())
}
