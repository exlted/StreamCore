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
FROM debian:latest AS runtime

RUN apt-get update && apt-get install -y libssl && apt clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /target/release/streamcore_youtube_chat_ingest /
COPY --from=builder /public/* /public/

EXPOSE 8082
CMD ["/streamcore_youtube_chat_ingest"]