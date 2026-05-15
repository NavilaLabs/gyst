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
      ZK_APPLICATION__SECURITY__AUTHENTICATION_SECRET: "changeme-use-a-real-secret"

      # SQLite database directory (must match the volume mount below).
      ZK_DATABASE__BASE_URI: "sqlite:///data/databases"
      ZK_DATABASE__DATABASES__ADMIN__NAME: "zeitrak_admin"

      ZK_PROJECT_ROOT: "/app"
      ZK_ENVIRONMENT: "production"

      # Public base URL — used to build invitation links sent by email.
      ZK_APPLICATION__BASE_URL: "https://zeitrak.example.com"
    volumes:
      # Persist SQLite database files across container restarts.
      - zeitrak_data:/data/databases

volumes:
  zeitrak_data:
```

### Environment variables

Configuration is layered: YAML files under `config/{environment}/` are merged first, then environment variables (prefixed `ZK_`, with `__` as the nested-key separator) override them at runtime.

#### Core (required)

| Variable | Description |
|---|---|
| `ZK_APPLICATION__SECURITY__AUTHENTICATION_SECRET` | HS256 secret used to sign JWT session tokens. Must be a long random string. |
| `ZK_DATABASE__BASE_URI` | Database base URI. For SQLite: `sqlite:///path/to/directory`. For PostgreSQL: `postgres://user:pass@host:5432`. |
| `ZK_PROJECT_ROOT` | Absolute path to the app root inside the container (default: `/app`). |
| `ZK_ENVIRONMENT` | `production` or `development`. Selects the config profile under `config/`. |

#### Database (optional)

| Variable | Default | Description |
|---|---|---|
| `ZK_DATABASE__DATABASES__ADMIN__NAME` | `zeitrak_admin` | Name of the shared admin database. |
| `ZK_DATABASE__DATABASES__TENANT__NAME_PREFIX` | `zeitrak_tenant_` | Prefix for per-workspace tenant databases. |
| `ZK_DATABASE__POOL__MAX_SIZE` | `20` | Maximum connections in the pool. |
| `ZK_DATABASE__POOL__MIN_SIZE` | `5` | Minimum idle connections in the pool. |
| `ZK_DATABASE__POOL__TIMEOUT_SECONDS` | `30` | Idle connection timeout in seconds. |

#### Application (optional)

| Variable | Default | Description |
|---|---|---|
| `ZK_APPLICATION__BASE_URL` | `http://localhost:8080` | Public base URL. Required for generating invitation links. |
| `ZK_APPLICATION__SECURITY__INVITE_ONLY` | `true` | When `true`, registration is restricted to invited users only. |

#### SMTP (optional — required for email features)

| Variable | Default | Description |
|---|---|---|
| `ZK_APPLICATION__SMTP__HOST` | `localhost` | SMTP server hostname. |
| `ZK_APPLICATION__SMTP__PORT` | `1025` | SMTP server port. |
| `ZK_APPLICATION__SMTP__USERNAME` | _(empty)_ | SMTP authentication username. |
| `ZK_APPLICATION__SMTP__PASSWORD` | _(empty)_ | SMTP authentication password. |
| `ZK_APPLICATION__SMTP__FROM_ADDRESS` | `noreply@zeitrak.app` | Sender address for outgoing emails. |
| `ZK_APPLICATION__SMTP__USE_TLS` | `true` | Set to `false` for plain SMTP without TLS (e.g. local MailHog). |

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
