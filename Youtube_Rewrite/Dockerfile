FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release
RUN ls -R

# second stage.
FROM gcr.io/distroless/cc-debian11 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/* /
COPY --from=builder /app/public/* /public/

EXPOSE 8082
CMD ["/streamcore_vstream_chat_ingest"]