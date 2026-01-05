# ============================================
# STAGE 1: Build Stage
# ============================================
FROM rust:alpine AS builder

# Install build dependencies including OpenSSL and CA Certs (for copying later)
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static ca-certificates

# Create a new empty project for dependency caching
WORKDIR /app

# Copy only dependency files first (Docker layer caching optimization)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer gets cached)
RUN cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null || true

# Remove the dummy build artifacts
RUN rm -rf src target/x86_64-unknown-linux-musl/release/deps/Pooling_application*

# Now copy the actual source code
COPY src ./src

# Build the actual application with static OpenSSL
ENV OPENSSL_STATIC=1
RUN cargo build --release --target x86_64-unknown-linux-musl

# Strip debug symbols to reduce binary size
RUN strip target/x86_64-unknown-linux-musl/release/Pooling_application

# ============================================
# STAGE 2: Runtime Stage (Scratch)
# ============================================
FROM scratch

# Copy CA certificates for HTTPS connections (from builder)
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the statically linked binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/Pooling_application /app

# Expose the application port
EXPOSE 3000

# Run the binary
ENTRYPOINT ["/app"]
