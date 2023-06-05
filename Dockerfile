FROM docker.io/library/rust:1.70.0-bullseye AS build

ARG DATABASE_URL

WORKDIR /app

RUN rustup component add rustfmt

COPY . /app

RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=build /app/target/release/exporter /

ENTRYPOINT ["./exporter"]
