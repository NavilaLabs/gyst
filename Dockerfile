# Stage 1: Builder
FROM rust:1 AS builder

WORKDIR /app

# Install dioxus-cli
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"
ENV ZK_DATABASE_BASE_URI=sqlite:///app/databases
ENV ZK_ADMIN_DATABASE_NAME=zeitrak_admin

COPY . .

# Build the whole workspace and then the Dioxus web package
RUN cd zeitrak && cargo build --release --bin admin-projection-daemon
RUN cd zeitrak && cargo build --release --bin tenant-projection-daemon
RUN cd zeitrak-presentation/gui && cargo build --release
RUN cd zeitrak-presentation/gui && dx build --package web --release

# ----------------------------------------------------------
# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built server binary and assets
COPY --from=builder /app/target/release/admin-projection-daemon .
COPY --from=builder /app/target/release/tenant-projection-daemon .
COPY --from=builder /app/zeitrak-presentation/gui/target/dx/web/release/web .

COPY --from=builder /app/config/ /app/config/

# Copy your entrypoint script (See note below!)
COPY --from=builder /app/entrypoint.sh /app/entrypoint.sh
# COPY --from=builder /app/server /app/server

RUN mkdir databases

EXPOSE 8080
ENV PORT=8080
ENV IP=0.0.0.0
ENV ZK_ENVIRONMENT=production
ENV ZK_PROJECT_ROOT="/app"
ENV ZK_DATABASE_BASE_URI=sqlite:///app/databases
ENV ZK_ADMIN_DATABASE_NAME=zeitrak_admin

RUN chmod +x /app/entrypoint.sh
ENTRYPOINT ["/app/entrypoint.sh"]
