FROM rust:1.81-slim AS planner
WORKDIR /usr/src/app
# We only need the Cargo.toml and Cargo.lock to plan dependencies
COPY Cargo.toml Cargo.toml
# Create a dummy Cargo.lock if it doesn't exist
RUN touch Cargo.lock

# Stage 1: Dependency caching layer
FROM rust:1.81-slim AS cacher
WORKDIR /usr/src/app
# First, copy the manifest files
COPY Cargo.toml Cargo.toml
# Create a fake src directory with minimal content needed for cargo
RUN mkdir -p src && \
    echo 'fn main() { println!("Dummy implementation"); }' > src/main.rs
# Build the dependencies without the actual source code
# This layer will be cached unless Cargo.toml changes
RUN cargo build --release

# Stage 2: Builder stage
FROM rust:1.81-slim AS builder
WORKDIR /usr/src/app
# Copy the manifests
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
# Copy the src directory
COPY src src/
# Copy the target directory from cacher stage to avoid rebuilding dependencies
COPY --from=cacher /usr/src/app/target target
# Build the application
RUN cargo build --release

# Stage 3: Final stage
FROM debian:bullseye-slim

# Install any runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -u 1001 -g 0 appuser

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/DDBP /usr/local/bin/app

# Set the user to run the application
USER appuser

# Command to run the executable
CMD ["/usr/local/bin/app"]
