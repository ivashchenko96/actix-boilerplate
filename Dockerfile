# Use the official Rust image as the build environment
FROM rust:stable as builder

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create src directory and dummy main.rs for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached unless Cargo files change)
RUN cargo build --release
RUN rm src/main.rs

# Copy the source code
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY locales ./locales

# Build the application
RUN touch src/main.rs
RUN cargo build --release

# Use a smaller base image for runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built application
COPY --from=builder /app/target/release/actix-boilerplate ./
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/config ./config
COPY --from=builder /app/locales ./locales

# Create non-root user
RUN useradd -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8000

CMD ["./actix-boilerplate"]
