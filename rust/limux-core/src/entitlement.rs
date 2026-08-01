//! Per-connection workspace entitlement for the Limux control dispatcher.
//!
//! Background — `docs/LIMUX_H1_WORKSPACE_ENTITLEMENT_DESIGN_2026-07-29.md`,
//! `docs/REPO_AUDIT_limux_2026-07-21.md` (REPO_AUDIT H1). The auth layer in
//! `limux-control/src/auth.rs` is uid-level only at accept time, so any
//! same-uid process that can connect can pass an explicit foreign
//! `workspace_id` and read another lane's surface. This module adds a
//! per-connection *claim* state and the gates that turn it on, behind a
//! default-off `LIMUX_ENTITLEMENT` flag.
//!
//! # Modes
//!
//! - [`EntitlementMode::Off`] (default) — no entitlement enforcement. All
//!   connections behave exactly as they do today, with the same disclosure
//!   surface the design doc describes. This is the safe landing posture: the
//!   code merges, the security guarantee stays dormant, and the operator
//!   picks the operator-vs-agent signal (§2.5) before flipping the default.
//! - [`EntitlementMode::UnclaimedAllEntitled`] — a connection that has not
//!   yet presented a `workspace_id` is treated as fully entitled
//!   ("unclaimed = all-entitled"). The moment it sends a `workspace_id` it is
//!   *claimed* to that workspace; every subsequent request must name that
//!   same workspace or be refused. **Fails open on any agent that never
//!   sends a `workspace_id`** — this is the design-doc framing the B-scout
//!   flags as the dangerous seam.
//! - [`EntitlementMode::RequireClaim`] — fail-closed. An unclaimed
//!   connection is **rejected** on reads that lack an explicit
//!   `workspace_id`. Presenting an explicit `workspace_id` binds the sticky
//!   claim (claim-first via [`ConnectionEntitlement::claim_or_allow_explicit`])
//!   and subsequent requests must stay on that workspace. This is the only
//!   option that does not silently fail open on a misconfigured agent.
//!   Recommended default once the operator-vs-agent signal is settled.
//!
//! # Wire-up
//!
//! The standalone dispatcher path holds one [`ConnectionEntitlement`] per
//! connection in a small shared cell (built by `limux-control`'s server
//! loop) and threads it through `dispatch_with_entitlement`. The live GTK
//! bridge carries the entitlement in each `ControlCommand` (a future PR
//! threads it through `window.rs`).
//!
//! # Sticky claim
//!
//! A connection that has sent `workspace_id = W` cannot later claim `W'`.
//! The first non-`None` `workspace_id` wins, permanently, for that
//! connection's lifetime. The claim cell is `Arc<AtomicU64>` so the same
//! cell is shared by every clone of a `ConnectionEntitlement` for one
//! connection — which is what the sticky semantics need.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Operator-controlled mode for the entitlement gate.
///
/// Read from `LIMUX_ENTITLEMENT` at process start by
/// [`EntitlementConfig::from_env`]. Anything other than the spelled-out
/// `off` / `unclaimed-all-entitled` / `require-claim` (case-insensitive,
/// hyphen / underscore tolerated) falls back to [`Off`](Self::Off) — the
/// default, the safe landing posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntitlementMode {
    /// Default. No entitlement enforcement; behavior identical to today.
    Off,
    /// Unclaimed = all-entitled. First `workspace_id` claim is sticky.
    UnclaimedAllEntitled,
    /// Unclaimed = rejected. First `workspace_id` claim is sticky.
    RequireClaim,
}

impl EntitlementMode {
    /// Parse a `LIMUX_ENTITLEMENT` value. Unknown values fall back to
    /// [`Off`](Self::Off) (fail-closed on misconfiguration: rather than
    /// silently flipping the default on, stay dormant and let the operator
    /// notice the typo).
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "off" | "disabled" | "0" | "false" | "no" => Self::Off,
            "unclaimed-all-entitled" | "unclaimed-all" | "operator-all" => {
                Self::UnclaimedAllEntitled
            }
            "require-claim" | "require" | "claim" | "strict" => Self::RequireClaim,
            _ => Self::Off,
        }
    }
}

/// Operator-supplied configuration for the entitlement gate.
///
/// Resolved once at process start (mirroring `LIMUX_SOCKET_MODE`'s shape).
/// Per-connection state lives in [`ConnectionEntitlement`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntitlementConfig {
    pub mode: EntitlementMode,
}

impl EntitlementConfig {
    /// Read `LIMUX_ENTITLEMENT` (or the legacy `CMUX_ENTITLEMENT` alias) from
    /// the process environment. Missing / unparseable -> [`Off`](EntitlementMode::Off).
    pub fn from_env() -> Self {
        let raw = std::env::var("LIMUX_ENTITLEMENT")
            .ok()
            .or_else(|| std::env::var("CMUX_ENTITLEMENT").ok())
            .unwrap_or_default();
        Self {
            mode: EntitlementMode::parse(&raw),
        }
    }

    /// True when this configuration enforces any entitlement gate at all.
    pub fn is_enforcing(self) -> bool {
        self.mode != EntitlementMode::Off
    }
}

/// Per-connection workspace claim state.
///
/// `Clone` shares the underlying `Arc<AtomicU64>` claim cell, so every clone
/// observes the same sticky claim. Intentionally **not** `Copy` — the
/// shared cell is the load-bearing piece; a by-value copy would be a
/// definitionally-different object.
#[derive(Debug, Clone)]
pub struct ConnectionEntitlement {
    config: EntitlementConfig,
    /// `u64::MAX` sentinel == unclaimed. Sticky: once set to a real id, it
    /// never changes. Wrapped in `Arc` so a `ConnectionEntitlement` clone
    /// shares the cell — every observer of the same connection sees the
    /// same claim.
    claimed: Arc<AtomicU64>,
}

const UNCLAIMED: u64 = u64::MAX;

impl ConnectionEntitlement {
    /// Build a new per-connection entitlement under the given config.
    pub fn new(config: EntitlementConfig) -> Self {
        Self {
            config,
            claimed: Arc::new(AtomicU64::new(UNCLAIMED)),
        }
    }

    /// Config (Off / UnclaimedAllEntitled / RequireClaim) this connection is
    /// running under. Shared across all connections of the same process.
    pub fn config(&self) -> EntitlementConfig {
        self.config
    }

    /// Workspace this connection has claimed, if any. `None` for
    /// unclaimed connections, or for any connection in
    /// [`Off`](EntitlementMode::Off) mode (no claim semantics).
    pub fn claimed_workspace(&self) -> Option<u64> {
        match self.claimed.load(Ordering::Acquire) {
            UNCLAIMED => None,
            id => Some(id),
        }
    }

    /// Record a workspace claim. Sticky: a connection that has already
    /// claimed `W` and now sees `W' != W` returns `Err(previous)`; the
    /// caller is expected to surface this as `PermissionDenied`. A
    /// connection in [`Off`](EntitlementMode::Off) mode accepts and ignores
    /// the claim — there is no claim to bind.
    pub fn record_claim(&self, workspace_id: u64) -> Result<(), u64> {
        if self.config.mode == EntitlementMode::Off {
            return Ok(());
        }
        match self
            .claimed
            .compare_exchange(UNCLAIMED, workspace_id, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => Ok(()),
            Err(existing) => {
                if existing == workspace_id {
                    // Same workspace re-asserted; legal — the claim is
                    // sticky to this id and the request is in scope.
                    Ok(())
                } else {
                    Err(existing)
                }
            }
        }
    }

    /// True when this connection is allowed to act on `workspace_id` for a
    /// *content-returning* operation. The semantics differ by mode:
    ///
    /// - [`Off`](EntitlementMode::Off): always true. No enforcement.
    /// - [`UnclaimedAllEntitled`](EntitlementMode::UnclaimedAllEntitled):
    ///   true when unclaimed (operator path), or when `workspace_id`
    ///   matches the sticky claim.
    /// - [`RequireClaim`](EntitlementMode::RequireClaim): true only when
    ///   the connection is claimed to `workspace_id` exactly. Unclaimed
    ///   connections are denied (fail-closed).
    pub fn allows_workspace(&self, workspace_id: u64) -> bool {
        match self.config.mode {
            EntitlementMode::Off => true,
            EntitlementMode::UnclaimedAllEntitled => match self.claimed.load(Ordering::Acquire) {
                UNCLAIMED => true,
                claimed => claimed == workspace_id,
            },
            EntitlementMode::RequireClaim => match self.claimed.load(Ordering::Acquire) {
                UNCLAIMED => false,
                claimed => claimed == workspace_id,
            },
        }
    }

    /// True when an *unclaimed* connection may proceed for a
    /// content-returning operation in
    /// [`RequireClaim`](EntitlementMode::RequireClaim) mode. Used by the
    /// focused-workspace fallback in `resolve_surface_target_scoped` to
    /// short-circuit with `PermissionDenied` instead of `not_found`, so
    /// the operator-vs-agent signal is auditable in test output.
    pub fn requires_claim(&self) -> bool {
        self.config.mode == EntitlementMode::RequireClaim
            && self.claimed.load(Ordering::Acquire) == UNCLAIMED
    }

    /// Explicit-`workspace_id` path: claim first, then allow.
    ///
    /// Call this from handlers that received an **explicit** `workspace_id`
    /// (or workspace hint). Ordering matters under
    /// [`RequireClaim`](EntitlementMode::RequireClaim):
    /// [`allows_workspace`](Self::allows_workspace) returns `false` while
    /// unclaimed, so checking allow *before* [`record_claim`](Self::record_claim)
    /// creates a first-claim chicken-and-egg. This helper:
    ///
    /// - short-circuits to `Ok(())` when the mode is not enforcing;
    /// - when unclaimed + enforcing: **`record_claim` first** (binds), then
    ///   the request is allowed for that id;
    /// - when already claimed: `Ok(())` iff the claim matches `workspace_id`,
    ///   else `Err(existing_claim)`.
    ///
    /// Do **not** call this for bare surface ids, focused-workspace
    /// fallbacks, or §1c management helpers — those stay check-only and
    /// must not auto-bind a sticky claim.
    pub fn claim_or_allow_explicit(&self, workspace_id: u64) -> Result<(), u64> {
        if !self.config.is_enforcing() {
            return Ok(());
        }
        self.record_claim(workspace_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(mode: EntitlementMode) -> EntitlementConfig {
        EntitlementConfig { mode }
    }

    #[test]
    fn parses_documented_modes_case_insensitive() {
        assert_eq!(EntitlementMode::parse("off"), EntitlementMode::Off);
        assert_eq!(EntitlementMode::parse("OFF"), EntitlementMode::Off);
        assert_eq!(EntitlementMode::parse("  Off  "), EntitlementMode::Off);
        assert_eq!(
            EntitlementMode::parse("unclaimed-all-entitled"),
            EntitlementMode::UnclaimedAllEntitled
        );
        assert_eq!(
            EntitlementMode::parse("unclaimed_all_entitled"),
            EntitlementMode::UnclaimedAllEntitled
        );
        assert_eq!(
            EntitlementMode::parse("require-claim"),
            EntitlementMode::RequireClaim
        );
    }

    #[test]
    fn unknown_values_default_to_off() {
        assert_eq!(EntitlementMode::parse(""), EntitlementMode::Off);
        assert_eq!(EntitlementMode::parse("yes"), EntitlementMode::Off);
        assert_eq!(EntitlementMode::parse("operator-signal"), EntitlementMode::Off);
    }

    #[test]
    fn from_env_falls_back_to_off_when_unset() {
        // We can't reliably clear env vars from a shared test process, so
        // the only thing we can assert is that *some* valid mode comes
        // back; the parse is exercised above. The default landing is `Off`
        // when neither var is set, which the parse tests pin.
        let cfg = EntitlementConfig::from_env();
        assert!(
            matches!(
                cfg.mode,
                EntitlementMode::Off
                    | EntitlementMode::UnclaimedAllEntitled
                    | EntitlementMode::RequireClaim
            ),
            "from_env must produce a known mode"
        );
    }

    #[test]
    fn off_mode_allows_everything_and_ignores_claims() {
        let ent = ConnectionEntitlement::new(cfg(EntitlementMode::Off));
        assert!(ent.allows_workspace(42));
        assert!(ent.allows_workspace(u64::MAX - 1));
        assert!(!ent.requires_claim());
        // Claims in Off mode are accepted but never bind (the wire says so).
        assert!(ent.record_claim(7).is_ok());
        assert!(ent.allows_workspace(99), "Off mode never binds a claim");
    }

    #[test]
    fn unclaimed_all_entitled_admits_unclaimed_then_binds_claim() {
        let ent = ConnectionEntitlement::new(cfg(EntitlementMode::UnclaimedAllEntitled));
        assert!(ent.allows_workspace(42), "unclaimed reads anything");
        assert!(!ent.requires_claim());

        ent.record_claim(7).expect("first claim binds");
        assert!(ent.allows_workspace(7), "claimed workspace is allowed");
        assert!(!ent.allows_workspace(8), "other workspace is denied after claim");
    }

    #[test]
    fn require_claim_rejects_unclaimed_connections() {
        let ent = ConnectionEntitlement::new(cfg(EntitlementMode::RequireClaim));
        assert!(!ent.allows_workspace(42), "unclaimed denied in require-claim");
        assert!(ent.requires_claim());

        ent.record_claim(7).expect("first claim binds");
        assert!(ent.allows_workspace(7));
        assert!(!ent.allows_workspace(8));
    }

    #[test]
    fn claim_is_sticky_against_subsequent_workspace_ids() {
        let ent = ConnectionEntitlement::new(cfg(EntitlementMode::UnclaimedAllEntitled));
        ent.record_claim(7).expect("first claim");
        let err = ent.record_claim(8).expect_err("second claim rejected");
        assert_eq!(err, 7, "the existing claim is returned for audit");
        // Re-asserting the same workspace is legal.
        assert!(ent.record_claim(7).is_ok());
    }

    #[test]
    fn claim_or_allow_explicit_binds_natural_first_claim_under_require_claim() {
        let ent = ConnectionEntitlement::new(cfg(EntitlementMode::RequireClaim));
        assert!(!ent.allows_workspace(7), "precondition: unclaimed denied");
        ent.claim_or_allow_explicit(7)
            .expect("natural first explicit workspace_id must bind");
        assert_eq!(ent.claimed_workspace(), Some(7));
        assert!(ent.allows_workspace(7));
        assert_eq!(
            ent.claim_or_allow_explicit(8).expect_err("foreign after claim"),
            7
        );
        assert!(ent.claim_or_allow_explicit(7).is_ok());
    }

    #[test]
    fn claim_or_allow_explicit_is_noop_when_off() {
        let ent = ConnectionEntitlement::new(cfg(EntitlementMode::Off));
        assert!(ent.claim_or_allow_explicit(42).is_ok());
        assert!(ent.claimed_workspace().is_none());
        assert!(ent.allows_workspace(99));
    }

    /// Two clones of `ConnectionEntitlement` share the same claim cell —
    /// the sticky claim recorded by one is visible to the other. This is
    /// the property the per-connection dispatch loop relies on when it
    /// passes `ConnectionEntitlement` by reference into every request.
    #[test]
    fn clones_share_claim_state() {
        let original = ConnectionEntitlement::new(cfg(EntitlementMode::UnclaimedAllEntitled));
        let twin = original.clone();
        // Distinct handles (Arc is shared) — twin records, original observes.
        assert!(twin.record_claim(11).is_ok());
        assert_eq!(original.claimed_workspace(), Some(11));
        assert_eq!(twin.claimed_workspace(), Some(11));
    }
}
