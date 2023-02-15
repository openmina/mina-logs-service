FROM rust:alpine AS build

RUN apk add musl-dev

COPY . .

RUN cargo install --path .

FROM alpine

COPY --from=build /usr/local/cargo/bin/mina-logs-service /usr/local/bin/

ENTRYPOINT [ "mina-logs-service" ]
