set dotenv-load := true
set dotenv-filename := ".env"
set shell := ["bash", "-uc"]

serve:
    just update && \
    cd /workspaces/loom/loom-presentation/gui/packages/web && \
    dx serve --fullstack --port 8080 --addr 0.0.0.0

project-admin:
    just update && \
    cargo run -p loom --bin admin-projection-daemon

project-tenant:
    just update && \
    cargo run -p loom --bin tenant-projection-daemon

watch-tw:
    just update && \
    cd /workspaces/loom/loom-presentation/gui/packages/ui && \
    deno run -A npm:@tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch

update:
    cargo update && cd loom-presentation/gui && cargo update

load-dev-env:
    [ -f .env ] && set -a && source .env && set +a
