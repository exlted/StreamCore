FROM rust:latest as builder

COPY . .

RUN cargo install --path .

# second stage.
FROM buildpack-deps:bookworm
COPY --from=builder /usr/local/cargo/bin/* /
CMD ["/streamcore_twitch_chat_ingest"]