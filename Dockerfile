# Builder stage
FROM rust:1.92-slim-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies (openssl is often needed for cargo dependencies like reqwest/sea-orm)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies
# Create a dummy project to cache compiled dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/gimme_backend*

# Build the actual application
COPY . .
# touch main.rs to force rebuild of the main package
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/gimme_backend .

# Expose the application port
EXPOSE 3000

# Run the application
CMD ["./gimme_backend"]
