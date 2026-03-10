//! Product-level versioning and checkpoint abstractions.

use serde::{Deserialize, Serialize};

use crate::{CheckpointName, RecordedAt, RetentionPolicy};

/// A concrete storage revision number surfaced as a product concept.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct VersionNumber(u64);

impl VersionNumber {
    /// Creates a version number.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric revision value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A request to create a stable checkpoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointRequest {
    name: CheckpointName,
    description: Option<String>,
}

impl CheckpointRequest {
    /// Creates a checkpoint request.
    #[must_use]
    pub fn new(name: CheckpointName, description: Option<String>) -> Self {
        Self { name, description }
    }

    /// Returns the requested checkpoint name.
    #[must_use]
    pub const fn name(&self) -> &CheckpointName {
        &self.name
    }

    /// Returns the optional description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// A user-visible named checkpoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    name: CheckpointName,
    version: VersionNumber,
    created_at: RecordedAt,
    description: Option<String>,
}

impl Checkpoint {
    /// Creates a new checkpoint record.
    #[must_use]
    pub fn new(name: CheckpointName, version: VersionNumber, description: Option<String>) -> Self {
        Self::new_at(name, version, RecordedAt::now(), description)
    }

    /// Creates a new checkpoint record with an explicit timestamp.
    #[must_use]
    pub fn new_at(
        name: CheckpointName,
        version: VersionNumber,
        created_at: RecordedAt,
        description: Option<String>,
    ) -> Self {
        Self {
            name,
            version,
            created_at,
            description,
        }
    }

    /// Returns the checkpoint name.
    #[must_use]
    pub const fn name(&self) -> &CheckpointName {
        &self.name
    }

    /// Returns the referenced version.
    #[must_use]
    pub const fn version(&self) -> VersionNumber {
        self.version
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> RecordedAt {
        self.created_at
    }

    /// Returns the optional description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// A compact checkpoint summary suited for inspection output.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointSummary {
    name: CheckpointName,
    version: VersionNumber,
}

impl CheckpointSummary {
    /// Creates a compact summary.
    #[must_use]
    pub const fn new(name: CheckpointName, version: VersionNumber) -> Self {
        Self { name, version }
    }

    /// Returns the checkpoint name.
    #[must_use]
    pub const fn name(&self) -> &CheckpointName {
        &self.name
    }

    /// Returns the referenced version number.
    #[must_use]
    pub const fn version(&self) -> VersionNumber {
        self.version
    }
}

/// Selects a restore or inspection target without exposing raw storage internals.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CheckpointSelector {
    /// Select by a named checkpoint.
    Named(CheckpointName),
    /// Select by a raw version number.
    Version(VersionNumber),
}

/// A request to restore the current head from a checkpoint or version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RestoreRequest {
    target: CheckpointSelector,
    retention_policy: RetentionPolicy,
}

impl RestoreRequest {
    /// Creates a restore request.
    #[must_use]
    pub fn new(target: CheckpointSelector) -> Self {
        Self {
            target,
            retention_policy: RetentionPolicy::conservative(),
        }
    }

    /// Returns the restore target selector.
    #[must_use]
    pub const fn target(&self) -> &CheckpointSelector {
        &self.target
    }

    /// Returns the retention policy applied to the restore flow.
    #[must_use]
    pub const fn retention_policy(&self) -> &RetentionPolicy {
        &self.retention_policy
    }

    /// Overrides the retention policy applied to the restore flow.
    #[must_use]
    pub fn with_retention_policy(mut self, retention_policy: RetentionPolicy) -> Self {
        self.retention_policy = retention_policy;
        self
    }
}

/// A product-level restore result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RestoreResult {
    target: CheckpointSelector,
    previous_version: VersionNumber,
    restored_version: VersionNumber,
    current_version: VersionNumber,
    pre_restore_checkpoint: Option<Checkpoint>,
}

impl RestoreResult {
    /// Creates a restore result.
    #[must_use]
    pub const fn new(
        target: CheckpointSelector,
        previous_version: VersionNumber,
        restored_version: VersionNumber,
        current_version: VersionNumber,
        pre_restore_checkpoint: Option<Checkpoint>,
    ) -> Self {
        Self {
            target,
            previous_version,
            restored_version,
            current_version,
            pre_restore_checkpoint,
        }
    }

    /// Returns the requested restore target.
    #[must_use]
    pub const fn target(&self) -> &CheckpointSelector {
        &self.target
    }

    /// Returns the version that was current before restore.
    #[must_use]
    pub const fn previous_version(&self) -> VersionNumber {
        self.previous_version
    }

    /// Returns the historical version that was restored.
    #[must_use]
    pub const fn restored_version(&self) -> VersionNumber {
        self.restored_version
    }

    /// Returns the new current head version created by restore.
    #[must_use]
    pub const fn current_version(&self) -> VersionNumber {
        self.current_version
    }

    /// Returns the checkpoint created before restore, if any.
    #[must_use]
    pub const fn pre_restore_checkpoint(&self) -> Option<&Checkpoint> {
        self.pre_restore_checkpoint.as_ref()
    }
}

/// A history entry representing a visible version in the product model.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VersionRecord {
    version: VersionNumber,
    recorded_at: RecordedAt,
    checkpoint: Option<CheckpointSummary>,
    summary: Option<String>,
}

impl VersionRecord {
    /// Creates a version record.
    #[must_use]
    pub fn new(
        version: VersionNumber,
        recorded_at: RecordedAt,
        checkpoint: Option<CheckpointSummary>,
        summary: Option<String>,
    ) -> Self {
        Self {
            version,
            recorded_at,
            checkpoint,
            summary,
        }
    }

    /// Returns the version number.
    #[must_use]
    pub const fn version(&self) -> VersionNumber {
        self.version
    }

    /// Returns the timestamp attached to the version.
    #[must_use]
    pub const fn recorded_at(&self) -> RecordedAt {
        self.recorded_at
    }

    /// Returns the checkpoint summary if the version is tagged.
    #[must_use]
    pub const fn checkpoint(&self) -> Option<&CheckpointSummary> {
        self.checkpoint.as_ref()
    }

    /// Returns the optional history summary.
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use super::*;

    #[test]
    fn checkpoint_request_preserves_description() {
        let request = CheckpointRequest::new(
            CheckpointName::try_from("pre-import").expect("valid checkpoint name"),
            Some("before import".to_string()),
        );

        assert_eq!(request.description(), Some("before import"));
    }

    #[test]
    fn version_record_can_reference_checkpoint_summary() {
        let summary = CheckpointSummary::new(
            CheckpointName::try_from("milestone-1").expect("valid checkpoint"),
            VersionNumber::new(7),
        );
        let record = VersionRecord::new(
            VersionNumber::new(7),
            RecordedAt::now(),
            Some(summary.clone()),
            None,
        );

        assert_eq!(record.checkpoint(), Some(&summary));
    }

    #[test]
    fn checkpoint_can_preserve_explicit_timestamp() {
        let timestamp = RecordedAt::new(UNIX_EPOCH);
        let checkpoint = Checkpoint::new_at(
            CheckpointName::try_from("restore-point").expect("valid checkpoint"),
            VersionNumber::new(3),
            timestamp,
            Some("before restore".to_string()),
        );

        assert_eq!(checkpoint.created_at(), timestamp);
    }

    #[test]
    fn restore_result_tracks_versions_and_auto_checkpoint() {
        let checkpoint = Checkpoint::new_at(
            CheckpointName::try_from("pre-restore-v7").expect("valid checkpoint"),
            VersionNumber::new(7),
            RecordedAt::new(UNIX_EPOCH),
            Some("automatic checkpoint before restore".to_string()),
        );
        let result = RestoreResult::new(
            CheckpointSelector::Version(VersionNumber::new(3)),
            VersionNumber::new(7),
            VersionNumber::new(3),
            VersionNumber::new(8),
            Some(checkpoint.clone()),
        );

        assert_eq!(result.previous_version(), VersionNumber::new(7));
        assert_eq!(result.restored_version(), VersionNumber::new(3));
        assert_eq!(result.current_version(), VersionNumber::new(8));
        assert_eq!(result.pre_restore_checkpoint(), Some(&checkpoint));
    }

    #[test]
    fn restore_request_defaults_to_conservative_retention() {
        let request = RestoreRequest::new(CheckpointSelector::Version(VersionNumber::new(4)));

        assert_eq!(request.retention_policy(), &RetentionPolicy::conservative());
    }

    #[test]
    fn restore_request_can_override_retention_policy() {
        let policy = RetentionPolicy::conservative()
            .with_pre_restore_checkpoint(crate::PreOperationCheckpointPolicy::Skip);
        let request = RestoreRequest::new(CheckpointSelector::Version(VersionNumber::new(4)))
            .with_retention_policy(policy.clone());

        assert_eq!(request.retention_policy(), &policy);
    }
}
