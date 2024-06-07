FROM rust:latest as builder

COPY . .

RUN cargo install --path .

# second stage.
FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/* /
CMD ["/streamcore_trovo_chat_ingest"]