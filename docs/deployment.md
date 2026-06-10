# Deployment

## Docker Compose

The recommended way to run Zeitrak in production is with Docker Compose. The stack consists of four services: a PostgreSQL database, the web server, and two background projection daemons.

### Prerequisites

- Docker 24+ and Docker Compose v2+
- The repository cloned locally (the image is built from source)

### Setup

1. Copy the example environment file and fill in the required values:

```bash
cp .env.example .env
```

2. Edit `.env`:

```dotenv
# JWT signing secret — required. Generate with: openssl rand -base64 64
ZK_APPLICATION__SECURITY__AUTHENTICATION_SECRET=<your-secret>

# PostgreSQL credentials
POSTGRES_USER=postgres
POSTGRES_PASSWORD=<your-password>

# Exposed port for the web service (default: 8080)
PORT=8080
```

### Starting

```bash
docker compose up -d
```

On first start the databases are automatically created and migrated. Navigate to `http://localhost:8080/setup` to complete the initial setup.

### Upgrading

```bash
docker compose build --no-cache
docker compose up -d
```

### Environment variables

Configuration is layered: YAML files under `config/{environment}/` are merged first, then environment variables (prefixed `ZK_`, with `__` as the nested-key separator) override them at runtime.

#### Required (set in `.env`)

| Variable | Description |
|---|---|
| `ZK_APPLICATION__SECURITY__AUTHENTICATION_SECRET` | HS256 secret used to sign JWT session tokens. Must be a long random string. |
| `POSTGRES_USER` | PostgreSQL superuser name. |
| `POSTGRES_PASSWORD` | PostgreSQL superuser password. |

#### Optional

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8080` | Host port mapped to the web service. |
| `WITH_LANDING` | `false` | Set to `true` to include the landing page in the web build. |

#### Database (optional overrides)

`ZK_DATABASE__BASE_URI` is derived automatically from `POSTGRES_USER` and `POSTGRES_PASSWORD` inside the compose file and does not need to be set manually.

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

### Services

| Service | Description |
|---|---|
| `postgres` | PostgreSQL 17 database. Data persisted in the `postgres_data` named volume. |
| `web` | Dioxus fullstack web server. Exposed on `PORT` (default `8080`). |
| `admin-projector` | Daemon that projects admin-scoped events into read-model tables. |
| `tenant-projector` | Daemon that projects tenant-scoped events into read-model tables. |

### Ports

Only port `8080` (or the value of `PORT`) needs to be exposed. Put a reverse proxy (nginx, Caddy, Traefik) in front for TLS termination or a custom domain.

### Logs

```bash
docker compose logs -f
docker compose logs -f web
docker compose logs -f admin-projector
docker compose logs -f tenant-projector
```
