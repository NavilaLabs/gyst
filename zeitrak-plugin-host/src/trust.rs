use dioxus_extism_host::TrustTag;

/// The entity that initiated the plugin install.
///
/// Passed to [`map_trust_tag`] together with the opaque [`TrustTag`] produced by
/// `dioxus-extism`'s signature verification to derive the zeitrak-specific
/// [`ZeitrakTrustTier`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Installer {
    /// A workspace administrator installed the plugin through the workspace
    /// settings UI or API.  Grants [`ZeitrakTrustTier::Tenant`] at most.
    WorkspaceAdmin,
    /// A server/instance administrator installed the plugin via the CLI or the
    /// instance-admin API.  Grants up to [`ZeitrakTrustTier::Instance`].
    InstanceAdmin,
    /// The boot-loader consumed the plugin from the configured trust directory
    /// and the bytes carry a valid Ed25519 signature against the trust root.
    /// Grants [`ZeitrakTrustTier::SignedInstance`].
    TrustRoot,
}

/// Context provided by the install path that `dioxus-extism` does not know about.
#[derive(Debug, Clone)]
pub struct InstallContext {
    /// Who initiated the install.
    pub installer: Installer,
}

/// zeitrak-specific trust tier derived from the dioxus-extism opaque [`TrustTag`]
/// and the [`InstallContext`].
///
/// `dioxus-extism` itself only produces the raw `TrustTag` (verified: bool,
/// `signer_key_id: Option<String>`).  All zeitrak policy decisions about what each
/// tier is allowed to do live exclusively in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ZeitrakTrustTier {
    /// Installed by a workspace admin.  Tenant-scope only: no filesystem, no
    /// outbound network, no admin-scope reads.
    Tenant,
    /// Installed by an instance admin via CLI or admin API.  Tenant + admin
    /// read-scope allowed.
    Instance,
    /// Ed25519-signed against the configured trust root.  Full access including
    /// admin writes.  Required for route-replace on core routes.
    SignedInstance,
}

/// Map an opaque `dioxus-extism` [`TrustTag`] + install context to a
/// zeitrak [`ZeitrakTrustTier`].
///
/// This is the single authoritative place where zeitrak interprets trust.
/// `dioxus-extism` learns nothing about these tiers.
#[must_use]
pub const fn map_trust_tag(tag: &TrustTag, ctx: &InstallContext) -> ZeitrakTrustTier {
    match (tag.verified, &ctx.installer) {
        (true, Installer::TrustRoot) => ZeitrakTrustTier::SignedInstance,
        (_, Installer::InstanceAdmin) => ZeitrakTrustTier::Instance,
        _ => ZeitrakTrustTier::Tenant,
    }
}

#[cfg(test)]
mod tests {
    use dioxus_extism_host::TrustTag;

    use super::*;

    fn tag(verified: bool) -> TrustTag {
        TrustTag {
            verified,
            signer_key_id: None,
        }
    }

    #[test]
    fn trust_root_with_verified_signature_yields_signed_instance() {
        let ctx = InstallContext {
            installer: Installer::TrustRoot,
        };
        assert_eq!(
            map_trust_tag(&tag(true), &ctx),
            ZeitrakTrustTier::SignedInstance
        );
    }

    #[test]
    fn trust_root_without_signature_falls_back_to_tenant() {
        let ctx = InstallContext {
            installer: Installer::TrustRoot,
        };
        assert_eq!(map_trust_tag(&tag(false), &ctx), ZeitrakTrustTier::Tenant);
    }

    #[test]
    fn instance_admin_verified_yields_instance() {
        let ctx = InstallContext {
            installer: Installer::InstanceAdmin,
        };
        assert_eq!(map_trust_tag(&tag(true), &ctx), ZeitrakTrustTier::Instance);
    }

    #[test]
    fn instance_admin_unverified_yields_instance() {
        let ctx = InstallContext {
            installer: Installer::InstanceAdmin,
        };
        assert_eq!(map_trust_tag(&tag(false), &ctx), ZeitrakTrustTier::Instance);
    }

    #[test]
    fn workspace_admin_yields_tenant() {
        let ctx = InstallContext {
            installer: Installer::WorkspaceAdmin,
        };
        assert_eq!(map_trust_tag(&tag(false), &ctx), ZeitrakTrustTier::Tenant);
    }
}
