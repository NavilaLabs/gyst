# Zeitrak Architektur-Review — Mai 2026

**Status**: Vorbedingung (Phase 1) für die Plugin-Plattform-Initiative (siehe Plan in `~/.claude/plans/`).
**Datum**: 2026-05-27
**Reviewer**: Claude (automatisierte Audit-Agenten, kuratiert)
**Scope**: Verifikation der Architektur-Konformität, Aggregate-Vollständigkeit, Auth, Event-Store-Integrität und technischer Schulden — als Baseline bevor Plugin-Hooks, Domain-Event-Bus und Plugin-Aggregate-Hosting eingebaut werden.

---

## Executive Summary

zeitrak ist in deutlich besserem Zustand als die ursprünglichen Plan-Annahmen unterstellten:

- **Architektur**: Onion/Ports&Adapters wird strikt eingehalten, `zeitrak-core` ist I/O-frei.
- **Aggregate-Vollständigkeit**: 8/8 Aggregate sind faktisch CRUD-komplett mit Projektoren und API-Endpunkten. `Permission` ist bewusst nur read-only/seed-getrieben. Einzig **Invitation** hat keine dedizierten Integration-Tests.
- **Auth & Authz**: JWT-Validation ist gehärtet (HS256-Whitelist, alg:none-Tests, Algorithm-Confusion-Tests). RBAC läuft konsistent über `AuthorizationService::require_permission`.
- **Multi-Tenancy**: Phantom-Type-basierte `Pool<Scope, State>` macht Scope-Crossing zur Compile-Zeit unmöglich. Tenant-DB ist per `workspace_id` im Pool gebunden.
- **Event-Store**: Persistenz ist sauber idempotent, Snapshots existieren, ProjectionRunner ist crash-resilient über `SqlCheckpoint`. **Wesentliche Lücke**: kein Event-Upcasting/Versionierung.

**Konsequenz für Phase 2 des Plugin-Plans**: Phase 2 (CRUD-Vervollständigung) ist **massiv kürzer als angenommen**. Nur Detail-Lücken (siehe P1/P2 unten) und das fehlende Event-Upcasting sind als Vorarbeit nötig.

---

## 1. Architektur-Konformität (Onion / Ports & Adapters)

**Status: konform.** Keine Verletzungen gefunden.

| Crate | Dependencies | Befund |
|---|---|---|
| `zeitrak-core` | `async-trait`, `chrono`, `serde`, `uuid`, `validator`, `eventually` | I/O-frei. Kein sqlx/axum/reqwest/tokio::fs. |
| `zeitrak-infrastructure` | `config`, `async-trait`, `chrono`, `tokio`, `url`, `figment` | Reine Trait-Definitionen (Ports). |
| `zeitrak-infrastructure-impl` | `sqlx`, `sea-query`, `sea-query-sqlx`, `jsonwebtoken`, `bcrypt`, `reqwest` | Adapter-Layer. Hängt korrekt an `zeitrak-core` + `zeitrak-infrastructure`. |
| `zeitrak` | Komposition aller Layer | Fasade, orchestriert Services. |

**Verifikation**: `grep` nach `sqlx`/`axum` in `zeitrak-core/src/` ist leer. Dependency-Inversion (Repository-Traits in core, Implementierungen in impl) ist überall durchgezogen.

---

## 2. Aggregate-Vollständigkeit

Auditiert: Activity, Timesheet, TimesheetTag (Tenant) sowie User, Workspace, WorkspaceRole, Permission, Invitation (Admin).

| Aggregate | Scope | Commands | Queries | Events | Projektor | API | Permissions | Tests |
|---|---|---|---|---|---|---|---|---|
| **Activity** | Tenant | Create, Update, Delete | find_by_id, find_all, list_all | Created, Updated, Deleted | ✓ | ✓ | ACTIVITY_CREATE/UPDATE/DELETE | ✓ (213 Z.) |
| **Timesheet** | Tenant | Start, Stop, Update, Reassign, Cancel, UpdateTime, CreateManual | find_by_id, find_all, filter_* | Started, Stopped, Updated, Reassigned, TimeUpdated, Cancelled | ✓ | ✓ | TIMESHEET_CREATE/UPDATE/CANCEL/EXPORT | ✓ (257 Z.) |
| **TimesheetTag** | Tenant | Create, Rename, Delete, Tag, Untag | find_by_id, find_all, list_by_timesheet | Created, Renamed, Deleted, TimesheetTagged, TimesheetUntagged | ✓ | ✓ | TAG_MANAGE | ✓ (258 Z.) |
| **User** | Admin | Create, UpdateSettings, RequestVerification, VerifyEmail | find_by_id, find_by_email, find_all, list_all | Created, SettingsUpdated, VerificationRequested, Verified | ✓ | ✓ | (auth-spezifisch) | ✓ (350 Z.) |
| **Workspace** | Admin | Create, AssignUserRole, RevokeUserRole, GrantUserPermission, RevokeUserPermission, UpdateSettings, RemoveMember | find_by_id, find_all, list_user_workspaces | Created, UserRoleAssigned/Revoked, UserPermissionGranted/Revoked, SettingsUpdated, UserRemoved | ✓ | ✓ | (implizit via Rolle) | ✓ (167 Z.) |
| **WorkspaceRole** | Admin | Create, GrantPermission, RevokePermission, Rename, Delete | find_by_id, find_all, list_by_workspace | Created, PermissionGranted/Revoked, Renamed, Deleted | ✓ | ✓ | ROLE_MANAGE | ✓ (281 Z.) |
| **Permission** | Admin | Create | find_by_id, find_all | Created | ✓ | ✓ (read-only) | — | ✓ (184 Z.) |
| **Invitation** | Admin | Create, Accept, Revoke | find_by_id, find_by_token, list_by_workspace, list_by_email | Created, Accepted, Revoked | ✓ | ✓ (send, list, accept, decline, revoke, register-and-accept) | MEMBER_INVITE | **fehlt** |

### Gap-Liste

Die ursprüngliche Plan-Annahme (User nur Settings-only, Permission nur Create, TimesheetTag Skeleton, Workspace ohne Commands) ist **überholt** — die Aggregate sind bereits ausgebaut.

**Real verbleibende Lücken**:
- **Invitation**: keine dedizierten Integration-Tests in `zeitrak-infrastructure-impl/tests/integration/`. Funktionalität wird nur indirekt durch andere Tests abgedeckt.
- **User**: kein `SoftDelete`/`Restore`. Kein `ChangePassword`-Flow getrennt von Settings (siehe Auth-Section).
- **Workspace**: kein `Delete`-Command. Aktuell keine Möglichkeit, einen Workspace zu löschen.
- **Activity**: kein `Restore` (nach `Delete`), kein `BulkUpdate`.
- **Timesheet**: kein hard `Delete` (nur `Cancel`).
- **Listing-Endpunkte**: keine einheitliche `FilterExpr`/`Page`/`Sort`-DSL — derzeit Aggregate-spezifische Filter-Funktionen (`filter_*`).

---

## 3. Authentifizierung & Autorisierung

### JWT-Validation
Status: **gehärtet**.
- Algorithm hardcoded auf `HS256` (`zeitrak/src/authentication.rs:56`).
- `alg:none` und Algorithm-Confusion durch Tests explizit abgewehrt (`zeitrak/tests/security/auth_tests.rs:179`, `:163`).
- Token-Lifetime: 1 h, mit Compile-Time-Assert gegen versehentliche Erhöhung (`zeitrak-infrastructure-impl/src/sea_query_sqlx/admin/authentication.rs:30`).
- **Kein Refresh-Token-Flow** für User-Sessions. SMTP OAuth2 nutzt verschlüsselte Refresh-Tokens (`zeitrak-infrastructure-impl/src/smtp/repository.rs`), aber Endnutzer müssen nach 1 h neu einloggen.
- **`kid`-Header wird nicht validiert** — relevant nur für Key-Rotation/Multi-Key.

### Authorization
- `RoleBasedPolicy` (`zeitrak/src/authorization.rs:45–127`) prüft:
  1. Admin-Bypass: Workspace-Rolle mit exaktem Namen `"admin"` (case-sensitive, hardcoded in Zeile 52).
  2. Permission-Grant via Role-Mapping ODER direkten User-Grant.
- **Hardcoded Admin-Rollenname** ist Hauptfindung — sollte konfigurierbar werden (env var) oder durch dedizierte `Permission::ADMIN_BYPASS` ersetzt.
- **Geschützte Endpunkte**: alle Mutationen (Activity Create/Update/Delete, Timesheet, Member, WorkspaceRole, Tag, Invitation).
- **Ungeschützte Read-Endpunkte**: `list_members`, `list_permissions`, `list_workspace_roles`, `list_activities`, `list_timesheets`, `get_invitation_by_token`. Begründung: Workspace-Membership reicht für Lesen. Vertretbar, aber Privacy-Modell sollte dokumentiert werden.
- **SQL-Injection**: parametrisierte Queries durchgängig, durch Tests verifiziert (`zeitrak/tests/security/authorization_tests.rs:207+`).

---

## 4. Multi-Tenancy-Isolation

Status: **stark, compile-time-sicher.**

- `Pool<Scope, State>` (`zeitrak-infrastructure-impl/src/sea_query_sqlx/infrastructure/pool.rs:60–114`) trägt Scope als Phantom-Type:
  ```rust
  pub struct Pool<Scope, State = StateDisconnected> {
      state: State,
      database_type: DatabaseType,
      scope: PhantomData<Scope>,
      tenant_id: Option<Uuid>,
  }
  ```
- Scopes: `ScopeDefault` (Bootstrap), `ScopeAdmin`, `ScopeTenant`. Cross-Scope-Aufrufe sind statisch unmöglich.
- Tenant-Pool wird via `connect_tenant(workspace_id)` an einen Workspace gebunden (`.../connect.rs:60–67`).
- Workspace-ID kommt in API-Handlern aus `session_workspace()` und wird explizit übergeben.

**Konsequenz für Plugins**: Plugin-Aggregate erben automatisch die Tenant-Isolation, wenn sie auf den Tenant-Pool zugreifen. Plugin-Storage-API muss diesen Pool-Typ respektieren.

---

## 5. Event-Store-Integrität

### Persistenz & Idempotenz
- Schema (`zeitrak-migrations/zeitrak-shared-migrations/src/lib.rs`):
  - `events`-Tabelle mit PK `(event_stream_id, version)` → idempotent.
  - `event_streams`-Tabelle mit PK `event_stream_id` → verhindert Duplikate.
- Snapshots existieren via `SnapshotRepository<A, P>` (`zeitrak-infrastructure-impl/src/snapshot.rs`) mit Index `(aggregate_type, aggregate_id, version)`.
- Projektoren nutzen `ON CONFLICT do_nothing()` (z.B. Activity-Projector Z. 53) → idempotente Replays.

### Snapshot-Strategie
- **Nicht konfiguriert**: weder `snapshot_every` per Aggregate noch eine zentrale Policy. Snapshot-Trigger sind aktuell opak in `eventually-any`. Für Plugin-Aggregate (Plan Phase 5) wird `snapshot_every` aus dem Manifest gesteuert — Pattern sollte auch auf Core-Aggregate übertragen werden.

### Event-Versionierung / Upcasting
- `schema_version`-Feld ist im DDL vorhanden, aber **wird nirgends gelesen oder validiert**. Schema-Änderungen würden zu stillem Deserialisierungs-Fehlverhalten führen.
- **Empfehlung**: Vor Plugin-Plattform-Launch ein `EventUpcaster`-Trait einführen, der Events anhand `schema_version` migriert. Plugin-Events sind sonst nicht zukunftssicher.

### ProjectionRunner
- Per Scope ein Daemon: `zeitrak/src/bin/tenant_projection_daemon.rs`, `admin_projection_daemon.rs`.
- Tenant-Daemon dispatcht sequenziell über `TenantProjector` (`zeitrak-infrastructure-impl/src/sea_query_sqlx/tenant/projectors.rs`) — FK-sichere Reihenfolge.
- Checkpoints via `SqlCheckpoint::new(pool, &name)` mit Namen wie `"tenant_projection_{workspace_id}"`. Crash-resilient.
- Unbekannte Event-Typen werden **silently ignoriert** im Projector-Dispatch. Mit Plugin-Plattform muss das robust bleiben (kein Crash bei unbekannten Plugin-Events im Core-Projector).

### Replay-Performance
- ✓ Safe durch Checkpoints — kein Full-Replay bei Restart.
- ⚠ `Root::rehydrate_from_state(0, user) // TODO`-Bug in `zeitrak-infrastructure-impl/src/sea_query_sqlx/admin/user/repositories.rs:1` zwingt User-Reads zum Full-Replay. **Muss behoben werden** (P1).

---

## 6. Test-Coverage-Übersicht

- 17 Integration-Tests über zwei Crates:
  - `zeitrak-infrastructure-impl/tests/integration/`: 12 (user, workspace, activity, timesheet, timesheet_tag, permission, workspace_role, smtp, database).
  - `zeitrak/tests/`: 5 (registration, security/auth, security/authorization).
- Keine `#[ignore]`-Tests.
- Im Domain-Layer 6 `unimplemented!("test stub")` in `ReadRepository`-Stubs — bewusste Test-Doubles, nicht Produktionscode.

### Gaps
- Invitation hat **keinen** dedizierten Integration-Test.
- Keine Snapshot-Recovery-Tests (verifizieren, dass Snapshots Full-Replay sparen).
- Keine Projection-Crash-Recovery-Tests (`SqlCheckpoint` Resume).
- Keine Multi-Workspace-Concurrency-Tests.

---

## 7. Technische Schulden & WIP

### Branches
- Lokal: nur `main` und der aktuelle Feature-Branch `claude/admiring-goldberg-mijsy`.
- **Kein** `multi-user`-Branch sichtbar (die Plan-Annahme ist überholt — Invitations sind bereits gemerged).
- **Kein** `desktop`/`i18n`-Branch sichtbar.

### TODOs
- 1 produktiver TODO: User-Aggregate Version-Bypass (siehe oben, Event-Store Section).

### i18n
- GUI nutzt `dioxus-i18n` mit `tid!()`-Macros. Backend hat keine i18n — API-Error-Messages sind hardcoded EN.

### Plugin-Skelett
- `zeitrak-core/src/plugin.rs`: `ZeitrakPlugin`-Trait (id/version/permissions) + `PluginRegistry` mit `register()` und `all_permissions()`. **Keine Runtime, keine Hooks, keine Extism-Integration**. Wird in Phase 3 des Plans durch `zeitrak-plugin-host` ersetzt/erweitert.

---

## 8. Priorisierte Findings-Liste

| ID | Prio | Bereich | Finding | Empfehlung | Aufwand |
|---|---|---|---|---|---|
| F1 | **P0** | Event-Store | Kein Event-Upcasting/Versionierung. `schema_version` im DDL, aber nirgends gelesen. | `EventUpcaster`-Trait + Dispatch im Repository-Load-Pfad. Vorbedingung für Plugin-Events. | 1–2 Tage |
| F2 | **P1** | Event-Store | `Root::rehydrate_from_state(0, user)`-Bug — User-Reads ignorieren Snapshot-Version. | Echte Version aus Snapshot-Repository ziehen. | 1–2 h |
| F3 | **P1** | Auth | Admin-Rollenname `"admin"` hardcoded und case-sensitive. | Env-Var `ADMIN_ROLE_NAME` ODER dedizierte `Permission::ADMIN_BYPASS`. Vor Plugin-Trust-Stufen. | 1–2 h |
| F4 | **P1** | Aggregate | Invitation hat keine Integration-Tests. | Test-Modul nach Pattern der anderen Aggregate. | 2–4 h |
| F5 | **P1** | Aggregate | Snapshot-Strategie pro Aggregate nicht konfiguriert. | `snapshot_every` als Aggregate-Metadatum, Policy ableiten. Vorbild für Plugin-Aggregate. | 0.5–1 Tag |
| F6 | **P1** | Auth | Kein Refresh-Token-Flow für User-Sessions; 1 h Hardcut. | Refresh-Endpoint hinzufügen ODER bewusst akzeptieren + dokumentieren. | 4–6 h |
| F7 | **P2** | Aggregate | User: kein SoftDelete/Restore, kein dedizierter ChangePassword. | Bei Plugin-Plattform-Bedarf nachziehen. | 0.5 Tag |
| F8 | **P2** | Aggregate | Workspace: kein `Delete`-Command. | Soft- oder Hard-Delete-Strategie wählen, dann ergänzen. | 0.5 Tag |
| F9 | **P2** | Aggregate | Keine einheitliche `FilterExpr`/`Page`/`Sort`-DSL. | `zeitrak-core/src/shared/query.rs` einführen, schrittweise migrieren. | 1–2 Tage |
| F10 | **P2** | Auth | `kid`-Header wird nicht validiert. | Defer bis Multi-Key/Rotation gebraucht wird. | — |
| F11 | **P2** | Auth | List-Endpunkte rufen kein `require_permission`. | Privacy-Modell explizit dokumentieren ODER `*.list`-Permissions einführen. | 0–0.5 Tag |
| F12 | **P2** | Backend | Keine i18n auf API-Errors. | Bei Bedarf nachziehen (post-Plugin). | — |
| F13 | **P2** | Tests | Keine Snapshot-/Projection-Recovery-Tests, keine Multi-Workspace-Concurrency. | Test-Suite ergänzen. | 1 Tag |

---

## 9. Konsequenzen für den Plugin-Plan

1. **Phase 2 schrumpft drastisch**. Die ursprüngliche Annahme einer großen CRUD-Vervollständigung trifft nicht zu. Stattdessen:
   - **P0-Vorbedingung**: F1 (Event-Upcasting) — kritisch für Plugin-Event-Schema-Evolution.
   - **P1-Bündel**: F2, F3, F4, F5, F6 vor oder parallel zur Plugin-Plattform.
   - **P2**: nach Bedarf während/nach Plugin-Plattform.

2. **Phase 7 (dioxus-extism)** ist unverändert in Tragweite — die generischen Manifest-Extensions / `call_plugin` / `HostCapability::Custom` / Route-`Replace` werden so gebraucht.

3. **Phase 3 (`zeitrak-plugin-host`)** profitiert davon, dass die Aggregate-Strukturen bereits sauber sind. Hook-Points können sofort in die existierenden Application-Services eingezogen werden (`zeitrak/src/{admin,tenant}/*.rs`).

4. **Multi-Tenancy-Garantien** bleiben durch das Plugin-System erhalten — Plugin-Code läuft in einer Sandbox; der Storage-API-Wrapper muss `Pool<ScopeTenant>` respektieren und darf nie auf `Pool<ScopeAdmin>` switchen ohne Trust-Check.

5. **Snapshots & Versionierung**: Das `snapshot_every` aus dem Plugin-Manifest (Plan Phase 5) sollte konsistent mit der Lösung von F5 sein — eine Strategie für Core und Plugins.
