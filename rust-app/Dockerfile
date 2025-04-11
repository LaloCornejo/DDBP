FROM rust:1.70 as builder
WORKDIR /usr/src/app

# Copy your Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock* ./

# This is a trick to cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy your actual source code
COPY . .

# Build your application
RUN cargo build --release

# Use a slim image for the final stage
FROM debian:bullseye-slim

# Install any runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/DDBP /usr/local/bin/app

# Run the application
CMD ["app"]
