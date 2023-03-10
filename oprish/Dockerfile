# syntax=docker/dockerfile:1
FROM rust:slim-buster as builder

WORKDIR /oprish

# Cargo workspace "necessities"
COPY ./pandemonium/Cargo.toml /pandemonium/Cargo.toml
COPY ./effis/Cargo.toml /effis/Cargo.toml
COPY ./cli/Cargo.toml /cli/Cargo.toml
RUN mkdir /pandemonium/src /effis/src /cli/src
RUN touch /pandemonium/src/main.rs /effis/src/main.rs /cli/src/main.rs

COPY ./Cargo.toml /Cargo.toml
COPY ./todel /todel
COPY ./oprish .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/target \
    cp /target/release/oprish /usr/local/bin/oprish

FROM debian:buster-slim

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES wait-for-it
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

ENV ROCKET_ADDRESS 0.0.0.0
ENV RUST_LOG debug

COPY --from=builder /usr/local/bin/oprish /bin/oprish

CMD ["wait-for-it", "mariadb:3306", "-t", "0", "--", "/bin/oprish"]
