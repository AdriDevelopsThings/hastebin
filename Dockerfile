FROM rust:alpine as build
WORKDIR /build

RUN apk add musl-dev

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src ./src

RUN cargo build --release

FROM scratch
WORKDIR /app

ENV PATH="$PATH:/app/bin"

COPY --from=build /build/target/release/hastebin /app/bin/hastebin

ENV LISTEN_ADDRESS=0.0.0.0:80
EXPOSE 80
ENV DATA_DIRECTORY=/data
VOLUME [ "/data" ]

CMD [ "/app/bin/hastebin" ]