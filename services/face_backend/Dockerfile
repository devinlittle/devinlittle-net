# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.99.1
ARG ALPINE_VERSION=3.24.1
ARG APP_NAME=face_backend

################################################################################
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app
COPY . .

RUN apk add --no-cache clang lld musl-dev openssl-libs-static git perl make curl protoc
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/app/target/,id=${APP_NAME} \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --release -p face_backend && \
cp ./target/release/$APP_NAME /bin/$APP_NAME

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

COPY --from=build /bin/face_backend/bin/
COPY --from=sqlx_build /usr/local/cargo/bin/sqlx /bin/sqlx

RUN mkdir -p /app/logs

USER appuser
ENV RUST_LOG=info
EXPOSE 3007
EXPOSE 50051

CMD ["/bin/face_backend"]
