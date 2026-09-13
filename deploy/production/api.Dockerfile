FROM rust:1.93-bookworm@sha256:7c4ae649a84014c467d79319bbf17ce2632ae8b8be123ac2fb2ea5be46823f31 AS build
WORKDIR /build
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
ENV CARGO_BUILD_JOBS=1
RUN cargo build --release --locked && strip target/release/promptark-api

FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home promptark && mkdir /data && chown promptark:promptark /data
COPY --from=build /build/target/release/promptark-api /usr/local/bin/promptark-api
USER 10001:10001
WORKDIR /data
EXPOSE 8787
CMD ["promptark-api"]
