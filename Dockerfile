# Stage 1: Builder
FROM rust:1.88-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Dependency caching: copy manifests first, build deps with dummy source
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "pub mod browser;\npub mod commands;\npub mod config;\npub mod diff;\npub mod error;\npub mod runner;\npub mod ui;" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p src/browser src/commands src/config src/diff src/error src/runner src/ui && \
    for dir in browser commands config diff error runner ui; do touch src/$dir/mod.rs; done && \
    cargo build --release && \
    rm -rf src

# Copy real source and rebuild
COPY src/ src/
RUN touch src/main.rs src/lib.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libnss3 \
    libnss3-tools \
    libxss1 \
    libatk-bridge2.0-0 \
    libgtk-3-0 \
    libgbm1 \
    libasound2 \
    libdrm2 \
    libxrandr2 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libcups2 \
    libpango-1.0-0 \
    libcairo2 \
    libatspi2.0-0 \
    fonts-liberation \
    fonts-noto-color-emoji \
    curl \
    unzip \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Download Chrome for Testing
RUN set -eux; \
    CHROME_JSON=$(curl -fsSL "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json"); \
    CHROME_VERSION=$(echo "$CHROME_JSON" | jq -r '.channels.Stable.version'); \
    CHROME_URL=$(echo "$CHROME_JSON" | jq -r '.channels.Stable.downloads.chrome[] | select(.platform == "linux64") | .url'); \
    CACHE_DIR="/root/.cache/xsnap/chromium/${CHROME_VERSION}"; \
    mkdir -p "$CACHE_DIR"; \
    curl -fsSL "$CHROME_URL" -o /tmp/chrome.zip; \
    unzip -q /tmp/chrome.zip -d /tmp/chrome; \
    mv /tmp/chrome/chrome-linux64/* "$CACHE_DIR/"; \
    rm -rf /tmp/chrome /tmp/chrome.zip; \
    chmod +x "$CACHE_DIR/chrome"

COPY --from=builder /build/target/release/xsnap /usr/local/bin/xsnap

ENTRYPOINT ["xsnap"]
