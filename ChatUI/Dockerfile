FROM rust:latest as builder

COPY . .

RUN cargo install --path .

# second stage.
FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/* /
COPY --from=builder /public/* /public/

EXPOSE 8080
CMD ["/chat_ui"]