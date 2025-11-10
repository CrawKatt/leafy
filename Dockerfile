# ============================
# Etapa 1: Build
# ============================
FROM rust:1.86 as builder

WORKDIR /build

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libasound2-dev \
        libssl-dev \
        libopus-dev \
        ffmpeg

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release || true

COPY src ./src

RUN cargo build --release


# ============================
# Etapa 2: Runtime
# ============================
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        libopus0 \
        ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Binario
COPY --from=builder /build/target/release/plantita_ayudante /app/plantita_ayudante

# Assets
COPY assets /app/assets

RUN chmod +x /app/plantita_ayudante

CMD ["/app/plantita_ayudante"]
