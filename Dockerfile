# Stage 1: Builder
FROM rust:1-trixie AS builder
WORKDIR /app
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install dioxus-cli --locked --root /.cargo --force
ENV PATH="/.cargo/bin:$PATH"
ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"

COPY . .

ARG WITH_LANDING=false

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cd zeitrak-presentation/gui && cargo fetch --locked

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cd zeitrak && cargo build --release --locked --features sqlite --bin admin-projection-daemon

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cd zeitrak && cargo build --release --locked --features sqlite --bin tenant-projection-daemon

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cd zeitrak-presentation/gui && \
    if [ "$WITH_LANDING" = "true" ]; then \
        dx build --package web --release --locked --features sqlite,landing; \
    else \
        dx build --package web --release --locked --features sqlite; \
    fi

# Stage 2: Web server
FROM debian:trixie AS web
RUN apt-get update \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/zeitrak-presentation/gui/target/dx/web/release/web .
COPY --from=builder /app/config/ /app/config/
EXPOSE 8080
ENV PORT=8080
ENV IP=0.0.0.0
ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"
CMD ["./server"]

# Stage 3: Admin projection daemon
FROM debian:trixie AS admin-projector
RUN apt-get update \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/admin-projection-daemon .
COPY --from=builder /app/config/ /app/config/
ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"
CMD ["./admin-projection-daemon"]

# Stage 4: Tenant projection daemon
FROM debian:trixie AS tenant-projector
RUN apt-get update \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/tenant-projection-daemon .
COPY --from=builder /app/config/ /app/config/
ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"
CMD ["./tenant-projection-daemon"]
