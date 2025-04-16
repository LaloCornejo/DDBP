# Stage 1: Builder stage - more efficient version
FROM rust:1.81-slim AS builder
WORKDIR /usr/src/app

# Copy only the files needed for build dependencies first
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY src ./src

# Build the application with the already-cached dependencies
RUN touch src/main.rs && cargo build --release

# Stage 2: Final runtime stage
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    bash \
    tini \
    && rm -rf /var/lib/apt/lists/*

# Create a script to run the app and keep container alive
RUN echo '#!/bin/bash\n\
echo "Starting application..."\n\
/usr/local/bin/DDBP 2>&1 | tee /tmp/app.log\n\
echo "Application exited. Container will remain running for log inspection."\n\
echo "To view logs, use: cat /tmp/app.log"\n\
echo "Press Ctrl+C to exit container"\n\
tail -f /dev/null' > /usr/local/bin/run-and-keep-alive.sh && \
    chmod +x /usr/local/bin/run-and-keep-alive.sh

# Create a non-root user
RUN useradd -u 1001 -g 0 appuser

# Copy only the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/DDBP /usr/local/bin/DDBP
RUN chmod +x /usr/local/bin/DDBP && \
    chown appuser:0 /usr/local/bin/DDBP /usr/local/bin/run-and-keep-alive.sh && \
    chmod 755 /usr/local/bin/DDBP /usr/local/bin/run-and-keep-alive.sh && \
    mkdir -p /tmp/logs && \
    chown -R appuser:0 /tmp/logs

# Set the user to run the application
USER appuser
WORKDIR /home/appuser

# Use tini as entrypoint to handle signals properly
ENTRYPOINT ["/usr/bin/tini", "--"]

# Command to run the executable and keep container alive
CMD ["/usr/local/bin/run-and-keep-alive.sh"]
