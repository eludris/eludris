# syntax=docker/dockerfile:1
FROM rust:slim-buster as builder

WORKDIR /pandemonium

# Cargo workspace "necessities"
COPY ./oprish/Cargo.toml /oprish/Cargo.toml
COPY ./effis/Cargo.toml /effis/Cargo.toml
COPY ./cli/Cargo.toml /cli/Cargo.toml
COPY ./codegen/Cargo.toml /codegen/Cargo.toml
RUN mkdir /oprish/src /effis/src /cli/src /codegen/src
RUN touch /oprish/src/main.rs /effis/src/main.rs /cli/src/main.rs /codegen/src/lib.rs

COPY ./Cargo.toml /Cargo.toml
COPY ./todel /todel
COPY ./pandemonium .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/target \
    cp /target/release/pandemonium /usr/local/bin/pandemonium

FROM debian:buster-slim

COPY ./wait-for-it.sh ./

ENV PANDEMONIUM_ADDRESS 0.0.0.0
ENV RUST_LOG debug

COPY --from=builder /usr/local/bin/pandemonium /bin/pandemonium

CMD ["./wait-for-it.sh", "mariadb:3306", "-t", "0", "--", "/bin/pandemonium"]
