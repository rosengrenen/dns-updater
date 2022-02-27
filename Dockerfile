FROM rust:1.59 as builder

WORKDIR /app

COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release


FROM ubuntu:20.04

WORKDIR /app

RUN apt update
RUN apt install -y libssl-dev
RUN apt install -y ca-certificates

COPY --from=builder /app/target/release/dns-updater /app/dns-updater

CMD /app/dns-updater
