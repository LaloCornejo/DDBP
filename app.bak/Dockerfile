# Builder stage
FROM rust:1.75-slim as builder
WORKDIR /usr/src/app

# Copy over your manifests and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build your application
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Install dependencies required by the binary at runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV MONGO_URI=mongodb://admin:password@10.89.0.3:27017,10.89.0.5:27017,10.89.0.7:27017/?replicaSet=rs0

# Copy only the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/DDBP /usr/local/bin/social-media-api

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/social-media-api

EXPOSE 8000

CMD ["/usr/local/bin/social-media-api"]
