FROM rust:latest as builder

WORKDIR /app

# create a new empty project
RUN cargo init

COPY . .
# build dependencies, when my source code changes, this build can be cached, we don't need to compile dependency again.
RUN cargo build
# remove the dummy build.
RUN cargo clean -p streamcore_twitch_chat_ingest

RUN cargo install --path .

# second stage.
FROM gcr.io/distroless/cc-debian11
COPY --from=builder /usr/local/cargo/bin/* /
CMD ["/streamcore_twitch_chat_ingest"]