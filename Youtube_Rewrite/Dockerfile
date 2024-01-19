FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

# second stage.
FROM gcr.io/distroless/cc-debian11 AS runtime
WORKDIR /app
COPY --from=builder /target/release/streamcore_youtube_chat_ingest /
COPY --from=builder /public/* /public/

EXPOSE 8082
CMD ["/streamcore_youtube_chat_ingest"]