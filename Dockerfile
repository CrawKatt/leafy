# Etapa 1: Build
FROM rust:1.86.0-nightly as builder

WORKDIR /usr/src/plantita_ayudante

# Copia los archivos de dependencias primero para aprovechar cache
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Compila el proyecto en modo release
RUN cargo build --release

# Etapa 2: Runtime
FROM debian:bookworm-slim

# Para ejecutar binarios de Rust
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copia el binario compilado
COPY --from=builder /usr/src/plantita_ayudante/target/release/plantita_ayudante .

# Comando por defecto
CMD ["./plantita_ayudante"]