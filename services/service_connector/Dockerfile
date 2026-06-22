# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.96.0
ARG ALPINE_VERSION=3.23.4
ARG APP_NAME=service_connector

################################################################################
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app
COPY . .

RUN apk add --no-cache clang lld musl-dev openssl-libs-static git perl make curl protoc
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

RUN --mount=type=cache,target=/app/target/,id=${APP_NAME} \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --release --bin ${APP_NAME} && \
cp ./target/release/$APP_NAME /bin/service_connector
################################################################################
FROM alpine:${ALPINE_VERSION} AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

COPY --from=build /bin/service_connector /bin/

RUN mkdir -p /app/logs

USER appuser
ENV RUST_LOG=info
EXPOSE 3006

CMD ["/bin/service_connector"]
