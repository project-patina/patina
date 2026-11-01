# The Play with friends relay, for anyone to run (docs/play-with-friends.md, "Running your own relay").
# Build context: the repository root (the workspace's manifests and lockfile):
#   docker build -f crates/patina-relay/Dockerfile -t patina-relay .
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p patina-relay \
    && cp target/release/patina-relay /patina-relay

FROM debian:bookworm-slim
COPY --from=build /patina-relay /usr/local/bin/patina-relay
ENV PORT=7000
EXPOSE 7000
USER nobody
CMD ["patina-relay"]
