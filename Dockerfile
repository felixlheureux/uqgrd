# Stage 1: Build Recipe
# FIXED: Pin to 'bookworm' so it matches the runtime OS version
FROM docker.io/library/rust:bookworm as builder

WORKDIR /app
COPY . .

# Build release binary
RUN cargo build --release

# Stage 2: Runtime Environment
# This matches the builder (Debian 12 Bookworm)
FROM docker.io/library/debian:bookworm-slim

# Install SSL certificates
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/uqgrd /usr/local/bin/uqgrd

# Create config folder
RUN mkdir -p /root/.config/uqgrd

# Use ENTRYPOINT so we can pass arguments (like 'credentials -s')
ENTRYPOINT ["uqgrd"]

# Default command
CMD ["start"]