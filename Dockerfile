# Stage 1: Build WASM
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /app

# Cache dependencies by copying manifests first
COPY Cargo.toml Cargo.lock ./
# Dummy lib.rs to pre-compile deps
RUN mkdir src && echo "pub fn dummy() {}" > src/lib.rs && \
    cargo build --release 2>/dev/null || true && \
    rm -rf src

COPY src ./src
RUN wasm-pack build --target web --out-dir www/pkg

COPY www/index.html www/
COPY www/app.js     www/

# Stage 2: Serve with nginx
FROM nginx:alpine

COPY --from=builder /app/www /usr/share/nginx/html

# nginx default config is fine; application/wasm is already in its mime.types
EXPOSE 80
