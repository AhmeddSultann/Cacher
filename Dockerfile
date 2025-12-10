# Multi-stage Dockerfile for production-grade deployment
# Stage 1: Build environment with cargo-chef for dependency caching
FROM rust:1.75-slim as chef
WORKDIR /app
RUN cargo install cargo-chef

# Stage 2: Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (cached layer)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build application
COPY . .
RUN cargo build --release --bin cacher

# Stage 5: Runtime (distroless for minimal attack surface)
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/cacher /app/cacher

# Expose gRPC port
EXPOSE 50051

# Run as non-root (distroless uses uid 65532 by default)
USER 65532:65532

# Start the server
ENTRYPOINT ["/app/cacher"]
