# Build image

FROM rust:latest as build

WORKDIR /tmp/build

COPY . .
RUN cargo build --release

# Executable image

FROM alpine:3.6

WORKDIR /opt/app

COPY --from=build /tmp/build/target/release/congenial-lamp .

ENV AERISWEATHER_CLIENT_ID
ENV AERISWEATHER_CLIENT_SECRET
ENV APIXU_API_KEY
ENV OPENWEATHERMAP_API_KEY
ENV WEATHERBIT_API_KEY
ENV ADDRESS

ENTRYPOINT [ "/opt/app/congenial-lamp" ]
CMD [ ]
