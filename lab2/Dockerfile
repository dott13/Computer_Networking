# Use the official Rust image as the base for building
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Install Diesel CLI
RUN cargo install diesel_cli --no-default-features --features sqlite

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install SQLite and other necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory for the runtime container
WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/lab2 /usr/src/app/lab2

# Copy Diesel CLI binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# Copy Diesel migrations
COPY --from=builder /usr/src/app/migrations /usr/src/app/migrations

# Expose ports for HTTP and WebSocket servers
EXPOSE 8080 8081

# Run Diesel migrations and start the application
CMD ["sh", "-c", "diesel migration run && ./lab2"]
