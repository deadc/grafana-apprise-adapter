# Stage 1: Builder
FROM rust:alpine as builder

RUN apk add --no-cache build-base

WORKDIR /usr/src/grafana-apprise-adapter

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build actual source
COPY ./src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM alpine:latest

# Security: Run as non-privileged user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

COPY --from=builder /usr/src/grafana-apprise-adapter/target/release/grafana-apprise-adapter /usr/local/bin/grafana-apprise-adapter

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=3s \
  CMD wget --no-verbose --tries=1 --spider http://localhost:5000/health || exit 1

CMD ["/usr/local/bin/grafana-apprise-adapter"]
