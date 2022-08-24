FROM rust:1.61 as build

RUN USER=root cargo new --bin eludris
WORKDIR /eludris

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/eludris*
RUN cargo build --release

FROM debian:buster-slim

COPY --from=build /eludris/target/release/eludris /usr/src/eludris

# Don't forget to also publish these ports in the docker-compose.yml file.
ARG REST_PORT=8000
ARG GATEWAY_PORT=9000

EXPOSE $REST_PORT
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT $REST_PORT

EXPOSE $GATEWAY_PORT
ENV GATEWAY_ADDRESS 0.0.0.0
ENV GATEWAY_PORT $GATEWAY_PORT
ENV RUST_LOG debug

CMD ["/usr/src/eludris"]
