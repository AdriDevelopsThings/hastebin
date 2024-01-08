FROM rust:alpine as build-backend
WORKDIR /build

RUN apk add musl-dev

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src ./src

RUN cargo build --release

FROM scratch
WORKDIR /app

ENV PATH="$PATH:/app/bin"

COPY --from=build-backend /build/target/release/axum-template /app/bin/axum-template

ENV LISTEN_ADDRESS=0.0.0.0:80
EXPOSE 80

CMD [ "/app/bin/axum-template" ]