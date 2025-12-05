# Multi-stage Dockerfile for optimized Rust builds on Render
# This caches dependencies separately from source code for faster rebuilds

# Stage 1: Build dependencies (cached layer)
FROM rust:1.75-slim as dependencies

WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy only dependency files first (for caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Stage 2: Build application
FROM rust:1.75-slim as builder

WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency cache from previous stage
COPY --from=dependencies /app/target /app/target
COPY --from=dependencies /usr/local/cargo /usr/local/cargo

# Copy the actual source code
COPY . .

# Build the application (dependencies are already cached)
RUN cargo build --release

# Stage 3: Runtime image (minimal size)
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/task-manager /app/task-manager

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Expose port (Render uses PORT env variable)
EXPOSE 10000

# Run the application
CMD ["/app/task-manager"]
