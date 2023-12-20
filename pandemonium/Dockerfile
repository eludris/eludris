# syntax=docker/dockerfile:1
FROM rust:slim-buster as builder

WORKDIR /pandemonium

# Cargo workspace "necessities"
COPY ./oprish/Cargo.toml /oprish/Cargo.toml
COPY ./effis/Cargo.toml /effis/Cargo.toml
COPY ./integration-tests/Cargo.toml /integration-tests/Cargo.toml
RUN mkdir /oprish/src /effis/src /integration-tests/src
RUN touch /oprish/src/main.rs /effis/src/main.rs /integration-tests/src/main.rs

COPY ./Cargo.toml /Cargo.toml
COPY ./Cargo.lock /Cargo.lock
COPY ./todel /todel
COPY ./codegen /codegen
COPY ./pandemonium .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/target \
    cp /target/release/pandemonium /usr/local/bin/pandemonium

FROM debian:buster-slim

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES wait-for-it
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

ENV PANDEMONIUM_ADDRESS 0.0.0.0
ENV RUST_LOG debug

COPY --from=builder /usr/local/bin/pandemonium /bin/pandemonium

CMD ["wait-for-it", "postgresql:5432", "-t", "0", "--", "/bin/pandemonium"]
