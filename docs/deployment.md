# Deployment

## Docker Compose

The recommended way to run Zeitrak in production is with Docker Compose. The application runs as a single container (web server + two background projection daemons launched by the entrypoint script).

### Prerequisites

- Docker 24+ and Docker Compose v2+
- A pre-built image (`docker build -t zeitrak:latest .` from the repo root)

### `docker-compose.yaml`

```yaml
services:
  zeitrak:
    image: zeitrak:latest
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      # Required: random secret used to sign session cookies.
      # Generate with: openssl rand -hex 32
      ZK_AUTHENTICATION_SECRET: "changeme-use-a-real-secret"

      # SQLite database directory (must match the volume mount below).
      ZK_DATABASE_BASE_URI: "sqlite:///data/databases"
      ZK_ADMIN_DATABASE_NAME: "zeitrak_admin"

      ZK_PROJECT_ROOT: "/app"
      ZK_ENVIRONMENT: "production"
    volumes:
      # Persist SQLite database files across container restarts.
      - zeitrak_data:/data/databases

volumes:
  zeitrak_data:
```

### Environment variables

| Variable | Required | Description |
|---|---|---|
| `ZK_AUTHENTICATION_SECRET` | Yes | Secret used to sign session tokens. Must be a long random string. |
| `ZK_DATABASE_BASE_URI` | Yes | SQLite base URI. The directory must be writable and persistent. |
| `ZK_ADMIN_DATABASE_NAME` | Yes | Name of the shared admin database (default: `zeitrak_admin`). |
| `ZK_PROJECT_ROOT` | Yes | Absolute path to the app root inside the container (default: `/app`). |
| `ZK_ENVIRONMENT` | No | `production` or `development`. Selects the config profile under `config/`. |

### Starting

```bash
docker compose up -d
```

On first start the admin database is automatically created and migrated. Navigate to `http://localhost:8080/setup` to complete the initial setup.

### Upgrading

```bash
docker compose pull   # or rebuild locally
docker compose up -d  # recreates the container; database files are preserved in the volume
```

### Ports

Only port `8080` needs to be exposed. Put a reverse proxy (nginx, Caddy, Traefik) in front if you need TLS termination or a custom domain.

### Logs

```bash
docker compose logs -f zeitrak
```

The projection daemons and the web server each write to stdout. All three processes are visible in the combined log stream.
