# Stage 1: Builder
FROM rust:1 AS builder

WORKDIR /app

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"

COPY . .

ARG WITH_LANDING=false

RUN cd zeitrak && cargo build --release --features postgres --bin admin-projection-daemon
RUN cd zeitrak && cargo build --release --features postgres --bin tenant-projection-daemon
RUN cd zeitrak-presentation/gui && \
    if [ "$WITH_LANDING" = "true" ]; then \
        dx build --package web --release --features server-postgres,landing; \
    else \
        dx build --package web --release --features server-postgres; \
    fi

# Stage 2: Web server
FROM debian:bookworm-slim AS web

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
FROM debian:bookworm-slim AS admin-projector

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
FROM debian:bookworm-slim AS tenant-projector

RUN apt-get update \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tenant-projection-daemon .
COPY --from=builder /app/config/ /app/config/

ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"

CMD ["./tenant-projection-daemon"]
