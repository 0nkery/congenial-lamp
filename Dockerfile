# Build image

FROM ekidd/rust-musl-builder:latest as build

COPY . .
RUN cargo build --release

# Executable image

FROM alpine:latest

COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/congenial-lamp \
    /usr/local/bin/

ENV AERISWEATHER_CLIENT_ID ""
ENV AERISWEATHER_CLIENT_SECRET ""
ENV APIXU_API_KEY ""
ENV OPENWEATHERMAP_API_KEY ""
ENV WEATHERBIT_API_KEY ""
ENV ADDRESS "0.0.0.0:8000"
ENV RUST_LOG "info"

CMD [ "/usr/local/bin/congenial-lamp" ]
