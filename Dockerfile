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

CMD ["/usr/src/eludris"]
