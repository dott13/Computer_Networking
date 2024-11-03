# Stage 1: Build the Rust application
FROM rust:1.82 AS builder

# Set the working directory
WORKDIR /usr/src/lab2

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a new empty shell project to cache dependencies
RUN mkdir src && echo "fn main() { println!(\"if you're seeing this, the build has failed\"); }" > src/main.rs

# Build the dependencies
RUN cargo build --release
RUN rm -f target/release/deps/lab2*

# Now copy the actual source code
COPY . .

# Build the actual application
RUN cargo build --release

# Stage 2: Create a lightweight image
FROM debian:buster-slim

# Set the working directory
WORKDIR /usr/src/lab2

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/lab2/target/release/lab2 .

# Expose the port your application listens on (adjust as needed)
EXPOSE 8080

# Command to run the application
CMD ["./lab2"]
