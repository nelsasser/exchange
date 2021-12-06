FROM rust:alpine
LABEL MAINTAINER="nick elsasser <nelsasser@colorado.edu>"

RUN cargo build --bin orderbook --release

WORKDIR /srv
COPY ./target/release/orderbook.exe /srv

CMD ["orderbook.exe"]