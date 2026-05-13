use sqlx::AnyPool;
/// Integration tests for `AuthorizationService`.
///
/// Each test creates its own [`TestFixture`] — a pair of freshly-migrated
/// isolated in-memory `SQLite` databases — seeds it with a known permission
/// matrix, and exercises the service.  Because every fixture is completely
/// independent, all tests run **concurrently** without `#[serial]`.
///
/// Security scenarios covered:
///   - Admin flag correctly detected via role name                ✓
///   - Non-admin roles do NOT grant admin                         ✓
///   - Unknown users are never treated as admin                   ✓
///   - Empty / zero-length `user_id` is safe                        ✓
///   - SQL injection in `user_id` returns false (parameterised)     ✓
///   - SQL injection in permission name returns false             ✓
///   - Permission via role grant                                  ✓
///   - Permission via direct user grant                           ✓
///   - Absent permission returns false                            ✓
///   - Role name "admin" is case-sensitive                        ✓
///   - Permission in workspace A does NOT grant access in B       ✓
///   - `require_admin` returns Ok for admins, Err for others      ✓
///   - `require_permission` admin bypass works                    ✓
///   - `require_permission` returns Err when permission absent    ✓
use zeitrak::{auth::CurrentUser, authorization::AuthorizationService};
use zeitrak_tests::TestFixture;

// ── Fixed test identifiers ────────────────────────────────────────────────────
// Deterministic UUIDs make tests reproducible and failure messages readable.

const WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000001";
const ADMIN_USER_ID: &str = "00000000-0000-0000-0000-000000000010";
const REGULAR_USER_ID: &str = "00000000-0000-0000-0000-000000000011";
const UNRELATED_USER_ID: &str = "00000000-0000-0000-0000-000000000012";
const ADMIN_ROLE_ID: &str = "00000000-0000-0000-0000-000000000020";
const VIEWER_ROLE_ID: &str = "00000000-0000-0000-0000-000000000021";
const PERMISSION_ID: &str = "00000000-0000-0000-0000-000000000030";
const PERMISSION_NAME: &str = "workspace.read";

// ── Helpers ───────────────────────────────────────────────────────────────────

fn admin_user() -> CurrentUser {
    CurrentUser {
        id: ADMIN_USER_ID.to_string(),
        email: "admin@test.com".to_string(),
    }
}

fn regular_user() -> CurrentUser {
    CurrentUser {
        id: REGULAR_USER_ID.to_string(),
        email: "user@test.com".to_string(),
    }
}

fn unknown_user() -> CurrentUser {
    CurrentUser {
        id: "00000000-0000-0000-0000-999999999999".to_string(),
        email: "ghost@test.com".to_string(),
    }
}

/// Seeds the admin database with a known, minimal permission matrix.
///
/// State after seeding:
/// - One workspace (`WORKSPACE_ID`)
/// - `ADMIN_USER_ID` → "admin" role (no explicit permissions)
/// - `REGULAR_USER_ID` → "viewer" role + direct grant of `PERMISSION_NAME`
/// - Viewer role has `PERMISSION_NAME` via role grant
async fn seed(pool: &AnyPool) {
    // 1. Workspace
    sqlx::query("INSERT INTO projections__workspaces (id, name) VALUES ($1, $2)")
        .bind(WORKSPACE_ID)
        .bind("Test Workspace")
        .execute(pool)
        .await
        .unwrap();

    // 2. Users
    for (id, name, email) in [
        (ADMIN_USER_ID, "Admin User", "admin@test.com"),
        (REGULAR_USER_ID, "Regular User", "user@test.com"),
    ] {
        sqlx::query(
            "INSERT INTO projections__users (id, name, email, password) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(name)
        .bind(email)
        .bind("$2b$12$placeholder_hash")
        .execute(pool)
        .await
        .unwrap();
    }

    // 3. Permission
    sqlx::query("INSERT INTO permissions (id, name) VALUES ($1, $2)")
        .bind(PERMISSION_ID)
        .bind(PERMISSION_NAME)
        .execute(pool)
        .await
        .unwrap();

    // 4. Workspace roles
    for (id, name) in [(ADMIN_ROLE_ID, "admin"), (VIEWER_ROLE_ID, "viewer")] {
        sqlx::query(
            "INSERT INTO projections__workspace_roles (id, workspace_id, name) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(WORKSPACE_ID)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
    }

    // 5. User ↔ role assignments
    for (user_id, role_id) in [
        (ADMIN_USER_ID, ADMIN_ROLE_ID),
        (REGULAR_USER_ID, VIEWER_ROLE_ID),
    ] {
        sqlx::query(
            "INSERT INTO projections__workspace_user_roles
             (workspace_id, user_id, workspace_role_id) VALUES ($1, $2, $3)",
        )
        .bind(WORKSPACE_ID)
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await
        .unwrap();
    }

    // 6. Role ↔ permission: viewer role gets PERMISSION_NAME
    sqlx::query(
        "INSERT INTO projections__workspace_role_permissions (workspace_role_id, permission_id)
         VALUES ($1, $2)",
    )
    .bind(VIEWER_ROLE_ID)
    .bind(PERMISSION_ID)
    .execute(pool)
    .await
    .unwrap();

    // 7. Direct user ↔ permission: regular_user also has it directly
    sqlx::query(
        "INSERT INTO projections__workspace_user_permissions
         (workspace_id, user_id, permission_id) VALUES ($1, $2, $3)",
    )
    .bind(WORKSPACE_ID)
    .bind(REGULAR_USER_ID)
    .bind(PERMISSION_ID)
    .execute(pool)
    .await
    .unwrap();
}

// ── is_admin ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn is_admin_returns_true_for_user_with_admin_role() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    assert!(
        AuthorizationService::is_admin_on(db.admin.as_ref(), ADMIN_USER_ID)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn is_admin_returns_false_for_user_with_non_admin_role() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    assert!(
        !AuthorizationService::is_admin_on(db.admin.as_ref(), REGULAR_USER_ID)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn is_admin_returns_false_for_unknown_user() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    assert!(
        !AuthorizationService::is_admin_on(db.admin.as_ref(), UNRELATED_USER_ID)
            .await
            .unwrap(),
        "a user absent from the system must not be treated as admin"
    );
}

#[tokio::test]
async fn is_admin_returns_false_for_empty_user_id() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    assert!(
        !AuthorizationService::is_admin_on(db.admin.as_ref(), "")
            .await
            .unwrap(),
        "empty user_id must not match any admin role"
    );
}

/// SQL injection via `user_id` must be neutralised by parameterised binds.
#[tokio::test]
async fn is_admin_sql_injection_in_user_id_is_neutralised() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;

    let injections = [
        "' OR '1'='1",
        "' OR 1=1--",
        "'; DROP TABLE projections__workspace_user_roles;--",
        "\\x00",
    ];

    for injection in &injections {
        let result = AuthorizationService::is_admin_on(db.admin.as_ref(), injection)
            .await
            .unwrap();
        assert!(
            !result,
            "SQL injection {injection:?} must not grant admin access"
        );
    }
}

/// Role name "admin" is case-sensitive: "Admin" must not satisfy the check.
#[tokio::test]
async fn is_admin_role_name_matching_is_case_sensitive() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;

    let capital_role_id = "00000000-0000-0000-0000-000000000099";

    sqlx::query(
        "INSERT INTO projections__users (id, name, email, password) VALUES ($1, $2, $3, $4)",
    )
    .bind(UNRELATED_USER_ID)
    .bind("Unrelated")
    .bind("unrelated@test.com")
    .bind("$2b$12$placeholder")
    .execute(db.admin.as_ref())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projections__workspace_roles (id, workspace_id, name) VALUES ($1, $2, $3)",
    )
    .bind(capital_role_id)
    .bind(WORKSPACE_ID)
    .bind("Admin") // capital A — must NOT match "admin"
    .execute(db.admin.as_ref())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projections__workspace_user_roles
         (workspace_id, user_id, workspace_role_id) VALUES ($1, $2, $3)",
    )
    .bind(WORKSPACE_ID)
    .bind(UNRELATED_USER_ID)
    .bind(capital_role_id)
    .execute(db.admin.as_ref())
    .await
    .unwrap();

    assert!(
        !AuthorizationService::is_admin_on(db.admin.as_ref(), UNRELATED_USER_ID)
            .await
            .unwrap(),
        "role named 'Admin' (capital A) must not satisfy the 'admin' check"
    );
}

// ── has_permission ────────────────────────────────────────────────────────────

#[tokio::test]
async fn has_permission_returns_true_via_role_grant() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    assert!(
        AuthorizationService::has_permission_on(
            db.admin.as_ref(),
            REGULAR_USER_ID,
            WORKSPACE_ID,
            PERMISSION_NAME,
        )
        .await
        .unwrap()
    );
}

#[tokio::test]
async fn has_permission_returns_true_via_direct_user_grant() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;

    // Insert a user with ONLY a direct grant (no role with the permission).
    let direct_only_id = "00000000-0000-0000-0000-000000000050";

    sqlx::query(
        "INSERT INTO projections__users (id, name, email, password) VALUES ($1, $2, $3, $4)",
    )
    .bind(direct_only_id)
    .bind("Direct Grant User")
    .bind("direct@test.com")
    .bind("$2b$12$placeholder")
    .execute(db.admin.as_ref())
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO projections__workspace_user_permissions
         (workspace_id, user_id, permission_id) VALUES ($1, $2, $3)",
    )
    .bind(WORKSPACE_ID)
    .bind(direct_only_id)
    .bind(PERMISSION_ID)
    .execute(db.admin.as_ref())
    .await
    .unwrap();

    assert!(
        AuthorizationService::has_permission_on(
            db.admin.as_ref(),
            direct_only_id,
            WORKSPACE_ID,
            PERMISSION_NAME,
        )
        .await
        .unwrap(),
        "direct permission grant must be sufficient without a role"
    );
}

#[tokio::test]
async fn has_permission_returns_false_for_user_with_no_grant() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    // admin_user has the admin role but that role has no permissions seeded.
    assert!(
        !AuthorizationService::has_permission_on(
            db.admin.as_ref(),
            ADMIN_USER_ID,
            WORKSPACE_ID,
            PERMISSION_NAME,
        )
        .await
        .unwrap(),
        "admin role has no permission grants in this fixture — must return false"
    );
}

#[tokio::test]
async fn has_permission_returns_false_for_unknown_permission_name() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    assert!(
        !AuthorizationService::has_permission_on(
            db.admin.as_ref(),
            REGULAR_USER_ID,
            WORKSPACE_ID,
            "permission.that.does.not.exist",
        )
        .await
        .unwrap()
    );
}

#[tokio::test]
async fn has_permission_returns_false_for_unknown_user() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    assert!(
        !AuthorizationService::has_permission_on(
            db.admin.as_ref(),
            UNRELATED_USER_ID,
            WORKSPACE_ID,
            PERMISSION_NAME,
        )
        .await
        .unwrap()
    );
}

/// SQL injection via the permission name must not grant access.
#[tokio::test]
async fn has_permission_sql_injection_in_permission_name_is_neutralised() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;

    let injections = [
        "' OR '1'='1",
        "' OR 1=1--",
        "workspace.read' OR 'x'='x",
        "'; DROP TABLE permissions;--",
    ];

    for injection in &injections {
        let result = AuthorizationService::has_permission_on(
            db.admin.as_ref(),
            ADMIN_USER_ID,
            WORKSPACE_ID,
            injection,
        )
        .await
        .unwrap();
        assert!(
            !result,
            "SQL injection in permission name {injection:?} must not grant access"
        );
    }
}

// ── cross-workspace isolation ─────────────────────────────────────────────────

/// A permission granted in `WORKSPACE_ID` must NOT be visible when checked
/// against a different workspace.  This is the core cross-tenant isolation
/// guarantee.
#[tokio::test]
async fn has_permission_is_scoped_to_workspace() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;

    let other_workspace = "00000000-0000-0000-ffff-000000000001";
    assert!(
        !AuthorizationService::has_permission_on(
            db.admin.as_ref(),
            REGULAR_USER_ID,
            other_workspace,
            PERMISSION_NAME,
        )
        .await
        .unwrap(),
        "permission in WORKSPACE_ID must not be visible in a different workspace"
    );
}

/// `require_permission` with the wrong workspace must return "forbidden" even
/// though the user genuinely holds the permission in their own workspace.
#[tokio::test]
async fn require_permission_cross_workspace_is_forbidden() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;

    let other_workspace = "00000000-0000-0000-ffff-000000000001";
    let err = AuthorizationService::require_permission_on(
        db.admin.as_ref(),
        &regular_user(),
        other_workspace,
        PERMISSION_NAME,
    )
    .await
    .expect_err("permission in a different workspace must be denied");
    assert!(
        err.to_string().contains("forbidden"),
        "error must say 'forbidden', got: {err}"
    );
}

// ── require_admin ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn require_admin_succeeds_for_admin_user() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    AuthorizationService::require_admin_on(db.admin.as_ref(), &admin_user())
        .await
        .expect("require_admin must return Ok for the admin user");
}

#[tokio::test]
async fn require_admin_fails_for_regular_user() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    let err = AuthorizationService::require_admin_on(db.admin.as_ref(), &regular_user())
        .await
        .expect_err("require_admin must fail for a non-admin user");
    assert!(
        err.to_string().contains("forbidden"),
        "error must say 'forbidden', got: {err}"
    );
}

#[tokio::test]
async fn require_admin_fails_for_unknown_user() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    let err = AuthorizationService::require_admin_on(db.admin.as_ref(), &unknown_user())
        .await
        .expect_err("require_admin must fail for a user not in the system");
    assert!(err.to_string().contains("forbidden"));
}

// ── require_permission ────────────────────────────────────────────────────────

/// Admin users bypass individual permission checks even without explicit grants.
#[tokio::test]
async fn require_permission_admin_bypasses_permission_check() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    AuthorizationService::require_permission_on(
        db.admin.as_ref(),
        &admin_user(),
        WORKSPACE_ID,
        PERMISSION_NAME,
    )
    .await
    .expect("admin must bypass permission checks even without an explicit grant");
}

#[tokio::test]
async fn require_permission_succeeds_for_user_with_permission() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    AuthorizationService::require_permission_on(
        db.admin.as_ref(),
        &regular_user(),
        WORKSPACE_ID,
        PERMISSION_NAME,
    )
    .await
    .expect("user with the permission must pass require_permission");
}

#[tokio::test]
async fn require_permission_fails_for_user_without_permission() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    let err = AuthorizationService::require_permission_on(
        db.admin.as_ref(),
        &regular_user(),
        WORKSPACE_ID,
        "permission.not.granted",
    )
    .await
    .expect_err("user without the permission must fail require_permission");
    assert!(
        err.to_string().contains("forbidden"),
        "error must say 'forbidden', got: {err}"
    );
}

#[tokio::test]
async fn require_permission_fails_for_unknown_user() {
    let db = TestFixture::setup().await;
    seed(db.admin.as_ref()).await;
    let err = AuthorizationService::require_permission_on(
        db.admin.as_ref(),
        &unknown_user(),
        WORKSPACE_ID,
        PERMISSION_NAME,
    )
    .await
    .expect_err("unknown user must fail require_permission");
    assert!(err.to_string().contains("forbidden"));
}
