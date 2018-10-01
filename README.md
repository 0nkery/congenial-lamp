# Что это?

Веб-сервис, отдающий агрегированный прогноз погоды по нескольким API.

# Использованные API

 * OpenWeatherMap
 * Aeris Weather
 * Apixu
 * WeatherBit

# Сборка

```shell
cargo build --release
```

# Запуск

Для запуска требуется установка переменных окружения с ключами для погодных API.

Переменные окружения:

  * `AERISWEATHER_CLIENT_ID`, `AERISWEATHER_CLIENT_SECRET` - параметры для Aeris Weather.
  * `APIXU_API_KEY` - ключ API Apixu.
  * `OPENWEATHERMAP_API_KEY` - ключ API OpenWeatherMap.
  * `WEATHERBIT_API_KEY` - ключ API WeatherBit.
  * `ADDRESS` - IP-адрес с портом, куда нужно забиндить сервер. По умолчанию `127.0.0.1:8088`.

Так же с помощью переменной окружения `RUST_LOG` можно настраивать многословность логгера.

# Docker

```shell
docker build -t forecast:latest .
docker run -e "AERISWEATHER_CLIENT_ID=client_id" -e "ADDRESS=0.0.0.0:8000" ... -d --rm -p 8000:8000 forecast:latest
curl http://localhost:8000/forecast/weekly/UK/London
```

# Endpoints

* `forecast/daily/{COUNTRY}/{CITY}/{DAY}` - прогноз на день для заданного города. Дата должна быть в формате YYYY-MM-DD,
например, `2018-10-02`.
* `forecast/weekly/{COUNTRY}/{CITY}` - прогноз на 5 дней для заданного города.
