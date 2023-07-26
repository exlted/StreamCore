FROM rust:latest as builder

COPY . .

RUN cargo install --path .

# second stage.
FROM gcr.io/distroless/cc-debian11
COPY --from=builder /usr/local/cargo/bin/* /
COPY --from=builder /public/* /public/

EXPOSE 8080
CMD ["/chat_ui"]