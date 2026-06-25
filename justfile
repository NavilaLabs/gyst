set dotenv-load := true
set dotenv-filename := ".env"
set shell := ["bash", "-uc"]

serve db="sqlite":
    just update && \
    cd /workspaces/zeitrak/zeitrak-presentation/gui/packages/web && \
    dx serve --fullstack --port 8080 --addr 0.0.0.0 --features {{db}}

serve-lp db="sqlite":
    just update && \
    cd /workspaces/zeitrak/zeitrak-presentation/gui/packages/web && \
    dx serve --fullstack --port 8080 --addr 0.0.0.0 --features {{db}},landing

project-admin db="sqlite":
    just update && \
    cargo run -p zeitrak --features {{db}} --bin admin-projection-daemon

project-tenant db="sqlite":
    just update && \
    cargo run -p zeitrak --features {{db}} --bin tenant-projection-daemon

watch-tw:
    just update && \
    cd /workspaces/zeitrak/zeitrak-presentation/gui/packages/ui && \
    deno run -A npm:@tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch

update:
    cargo update && cd zeitrak-presentation/gui && cargo update

test-postgres:
    cargo test -p zeitrak-infrastructure-impl --features postgres --test integration_infrastructure -- database::postgres

load-dev-env:
    [ -f .env ] && set -a && source .env && set +a
