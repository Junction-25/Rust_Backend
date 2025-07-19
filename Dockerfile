FROM rust:latest as builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/real-estate-recommender /usr/local/bin/real-estate-recommender

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

WORKDIR /app

EXPOSE 8080

CMD ["real-estate-recommender"]
