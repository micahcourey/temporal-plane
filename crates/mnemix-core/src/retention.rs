//! Conservative retention and cleanup policies.

use serde::{Deserialize, Serialize};

/// Describes how aggressively cleanup may remove historical versions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CleanupMode {
    /// Cleanup must always be explicit and conservative.
    ExplicitOnly,
    /// Cleanup may prune unprotected history after explicit review.
    AllowPrune,
}

/// Describes how checkpoints are protected during cleanup.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CheckpointProtection {
    /// Every named checkpoint is protected from routine cleanup.
    ProtectAll,
    /// Only checkpoints selected by policy are protected.
    ProtectNamedOnly,
}

/// Describes what should happen before a potentially destructive operation begins.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PreOperationCheckpointPolicy {
    /// Create a new automatic checkpoint using the provided prefix.
    AutoCreate {
        /// Prefix used when generating an automatic pre-operation checkpoint name.
        prefix: String,
    },
    /// Require the caller to create a checkpoint explicitly.
    RequireCallerProvided,
    /// Skip automatic pre-operation checkpoint creation.
    Skip,
}

/// Backwards-compatible alias for cleanup-specific policy naming.
pub type PreCleanupCheckpointPolicy = PreOperationCheckpointPolicy;

/// A storage-agnostic retention policy with conservative defaults.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    recent_versions_to_keep: u64,
    minimum_age_days: u16,
    checkpoint_protection: CheckpointProtection,
    cleanup_mode: CleanupMode,
    pre_cleanup_checkpoint: PreOperationCheckpointPolicy,
    pre_restore_checkpoint: PreOperationCheckpointPolicy,
    pre_import_checkpoint: PreOperationCheckpointPolicy,
    pre_optimize_checkpoint: PreOperationCheckpointPolicy,
    delete_unverified: bool,
    error_if_tagged_old_versions: bool,
}

impl RetentionPolicy {
    /// Creates a conservative default retention policy.
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            recent_versions_to_keep: 50,
            minimum_age_days: 30,
            checkpoint_protection: CheckpointProtection::ProtectAll,
            cleanup_mode: CleanupMode::ExplicitOnly,
            pre_cleanup_checkpoint: PreOperationCheckpointPolicy::AutoCreate {
                prefix: "pre-cleanup".to_string(),
            },
            pre_restore_checkpoint: PreOperationCheckpointPolicy::AutoCreate {
                prefix: "pre-restore".to_string(),
            },
            pre_import_checkpoint: PreOperationCheckpointPolicy::AutoCreate {
                prefix: "pre-import".to_string(),
            },
            pre_optimize_checkpoint: PreOperationCheckpointPolicy::AutoCreate {
                prefix: "pre-optimize".to_string(),
            },
            delete_unverified: false,
            error_if_tagged_old_versions: true,
        }
    }

    /// Returns the number of recent versions to preserve.
    #[must_use]
    pub const fn recent_versions_to_keep(&self) -> u64 {
        self.recent_versions_to_keep
    }

    /// Returns the minimum age threshold before old versions may be pruned.
    #[must_use]
    pub const fn minimum_age_days(&self) -> u16 {
        self.minimum_age_days
    }

    /// Returns the checkpoint protection mode.
    #[must_use]
    pub const fn checkpoint_protection(&self) -> CheckpointProtection {
        self.checkpoint_protection
    }

    /// Returns the cleanup mode.
    #[must_use]
    pub const fn cleanup_mode(&self) -> CleanupMode {
        self.cleanup_mode
    }

    /// Returns the pre-cleanup checkpoint behavior.
    #[must_use]
    pub fn pre_cleanup_checkpoint(&self) -> &PreOperationCheckpointPolicy {
        &self.pre_cleanup_checkpoint
    }

    /// Returns the pre-restore checkpoint behavior.
    #[must_use]
    pub fn pre_restore_checkpoint(&self) -> &PreOperationCheckpointPolicy {
        &self.pre_restore_checkpoint
    }

    /// Returns the pre-import checkpoint behavior.
    #[must_use]
    pub fn pre_import_checkpoint(&self) -> &PreOperationCheckpointPolicy {
        &self.pre_import_checkpoint
    }

    /// Returns the pre-optimize checkpoint behavior.
    #[must_use]
    pub fn pre_optimize_checkpoint(&self) -> &PreOperationCheckpointPolicy {
        &self.pre_optimize_checkpoint
    }

    /// Returns whether unverified files may be removed during pruning.
    #[must_use]
    pub const fn delete_unverified(&self) -> bool {
        self.delete_unverified
    }

    /// Returns whether tagged old versions should raise an error during pruning.
    #[must_use]
    pub const fn error_if_tagged_old_versions(&self) -> bool {
        self.error_if_tagged_old_versions
    }

    /// Overrides the cleanup mode.
    #[must_use]
    pub fn with_cleanup_mode(mut self, cleanup_mode: CleanupMode) -> Self {
        self.cleanup_mode = cleanup_mode;
        self
    }

    /// Overrides the prune age threshold.
    #[must_use]
    pub fn with_minimum_age_days(mut self, minimum_age_days: u16) -> Self {
        self.minimum_age_days = minimum_age_days;
        self
    }

    /// Overrides the pre-restore checkpoint behavior.
    #[must_use]
    pub fn with_pre_restore_checkpoint(mut self, policy: PreOperationCheckpointPolicy) -> Self {
        self.pre_restore_checkpoint = policy;
        self
    }

    /// Overrides the pre-import checkpoint behavior.
    #[must_use]
    pub fn with_pre_import_checkpoint(mut self, policy: PreOperationCheckpointPolicy) -> Self {
        self.pre_import_checkpoint = policy;
        self
    }

    /// Overrides the pre-optimize checkpoint behavior.
    #[must_use]
    pub fn with_pre_optimize_checkpoint(mut self, policy: PreOperationCheckpointPolicy) -> Self {
        self.pre_optimize_checkpoint = policy;
        self
    }

    /// Overrides whether unverified files may be removed during pruning.
    #[must_use]
    pub fn with_delete_unverified(mut self, delete_unverified: bool) -> Self {
        self.delete_unverified = delete_unverified;
        self
    }

    /// Overrides whether tagged old versions should block pruning.
    #[must_use]
    pub fn with_error_if_tagged_old_versions(mut self, error_if_tagged_old_versions: bool) -> Self {
        self.error_if_tagged_old_versions = error_if_tagged_old_versions;
        self
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self::conservative()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_conservative() {
        let policy = RetentionPolicy::default();

        assert_eq!(policy.cleanup_mode(), CleanupMode::ExplicitOnly);
        assert_eq!(
            policy.checkpoint_protection(),
            CheckpointProtection::ProtectAll
        );
        assert_eq!(policy.recent_versions_to_keep(), 50);
        assert_eq!(policy.minimum_age_days(), 30);
        assert!(policy.error_if_tagged_old_versions());
        assert!(!policy.delete_unverified());
    }

    #[test]
    fn conservative_policy_creates_pre_cleanup_checkpoint() {
        let policy = RetentionPolicy::conservative();

        assert_eq!(
            policy.pre_cleanup_checkpoint(),
            &PreOperationCheckpointPolicy::AutoCreate {
                prefix: "pre-cleanup".to_string(),
            }
        );
    }

    #[test]
    fn conservative_policy_creates_restore_and_optimize_checkpoints() {
        let policy = RetentionPolicy::conservative();

        assert_eq!(
            policy.pre_restore_checkpoint(),
            &PreOperationCheckpointPolicy::AutoCreate {
                prefix: "pre-restore".to_string(),
            }
        );
        assert_eq!(
            policy.pre_optimize_checkpoint(),
            &PreOperationCheckpointPolicy::AutoCreate {
                prefix: "pre-optimize".to_string(),
            }
        );
    }

    #[test]
    fn policy_builders_override_prune_flags() {
        let policy = RetentionPolicy::conservative()
            .with_delete_unverified(true)
            .with_error_if_tagged_old_versions(false);

        assert!(policy.delete_unverified());
        assert!(!policy.error_if_tagged_old_versions());
    }
}
