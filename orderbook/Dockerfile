FROM rust:latest as builder
LABEL MAINTAINER="nick elsasser <nelsasser@colorado.edu>"

RUN rustup toolchain install nightly
RUN rustup component add rustfmt --toolchain nightly-x86_64-unknown-linux-gnu

RUN USER=root cargo new --bin orderbook
WORKDIR ./orderbook
COPY ./Cargo.toml ./Cargo.toml
RUN cargo +nightly build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/orderbook*
RUN cargo +nightly build --bin orderbook --release

FROM debian:buster-slim

WORKDIR ./srv/orderbook

COPY --from=builder ./orderbook/target/release/orderbook* ./

CMD ["./orderbook"]