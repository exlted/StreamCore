FROM rust:latest as builder

WORKDIR /app

COPY . .

RUN cargo install --path .

# second stage.
FROM gcr.io/distroless/cc-debian11
COPY --from=builder /usr/local/cargo/bin/* /
CMD ["/streamcore_trovo_chat_ingest"]