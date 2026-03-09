//! Advanced branch-aware storage contracts.

use serde::{Deserialize, Serialize};

use crate::{CoreError, RecordedAt, VersionNumber};

const MAX_BRANCH_NAME_LEN: usize = 128;

fn validate_branch_name(value: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyValue {
            field: "branch_name",
        });
    }

    let actual = trimmed.chars().count();
    if actual > MAX_BRANCH_NAME_LEN {
        return Err(CoreError::TooLong {
            field: "branch_name",
            max: MAX_BRANCH_NAME_LEN,
            actual,
        });
    }

    if trimmed == "main" {
        return Err(CoreError::ReservedValue {
            field: "branch_name",
            value: trimmed.to_owned(),
        });
    }

    if trimmed.starts_with('/') || trimmed.ends_with('/') || trimmed.contains("//") {
        return Err(CoreError::InvalidCharacter {
            field: "branch_name",
            character: '/',
        });
    }

    if trimmed.contains("..") {
        return Err(CoreError::InvalidCharacter {
            field: "branch_name",
            character: '.',
        });
    }

    if trimmed.contains('\\') {
        return Err(CoreError::InvalidCharacter {
            field: "branch_name",
            character: '\\',
        });
    }

    if std::path::Path::new(trimmed)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("lock"))
    {
        return Err(CoreError::InvalidCharacter {
            field: "branch_name",
            character: '.',
        });
    }

    for character in trimmed.chars() {
        if character == '/' {
            continue;
        }
        if !(character.is_alphanumeric()
            || character == '.'
            || character == '-'
            || character == '_')
        {
            return Err(CoreError::InvalidCharacter {
                field: "branch_name",
                character,
            });
        }
    }

    Ok(trimmed.to_owned())
}

/// A validated branch name for advanced storage workflows.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct BranchName(String);

impl BranchName {
    /// Creates a validated branch name.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the branch name is blank, too long, or
    /// contains unsupported path characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        Ok(Self(validate_branch_name(&value.into())?))
    }

    /// Returns the validated branch name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for BranchName {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for BranchName {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<str> for BranchName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for BranchName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The visible lifecycle state of a storage branch.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BranchStatus {
    /// The branch is available for active work.
    Active,
    /// The branch was intentionally abandoned.
    Abandoned,
    /// The branch was merged into another line of history.
    Merged,
}

/// A visible record describing a storage branch.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BranchRecord {
    name: BranchName,
    base_version: VersionNumber,
    created_at: RecordedAt,
    status: BranchStatus,
}

impl BranchRecord {
    /// Creates a branch record.
    #[must_use]
    pub const fn new(
        name: BranchName,
        base_version: VersionNumber,
        created_at: RecordedAt,
        status: BranchStatus,
    ) -> Self {
        Self {
            name,
            base_version,
            created_at,
            status,
        }
    }

    /// Returns the branch name.
    #[must_use]
    pub const fn name(&self) -> &BranchName {
        &self.name
    }

    /// Returns the version the branch was created from.
    #[must_use]
    pub const fn base_version(&self) -> VersionNumber {
        self.base_version
    }

    /// Returns the timestamp the branch was created at.
    #[must_use]
    pub const fn created_at(&self) -> RecordedAt {
        self.created_at
    }

    /// Returns the visible lifecycle state.
    #[must_use]
    pub const fn status(&self) -> BranchStatus {
        self.status
    }
}

/// A request to create a new storage branch.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BranchRequest {
    name: BranchName,
    base_version: Option<VersionNumber>,
}

impl BranchRequest {
    /// Creates a branch request using the latest visible version by default.
    #[must_use]
    pub fn new(name: BranchName) -> Self {
        Self {
            name,
            base_version: None,
        }
    }

    /// Returns the requested branch name.
    #[must_use]
    pub const fn name(&self) -> &BranchName {
        &self.name
    }

    /// Returns the optional base version override.
    #[must_use]
    pub const fn base_version(&self) -> Option<VersionNumber> {
        self.base_version
    }

    /// Overrides the base version for the new branch.
    #[must_use]
    pub fn with_base_version(mut self, base_version: VersionNumber) -> Self {
        self.base_version = Some(base_version);
        self
    }
}

/// A visible branch listing result.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BranchListResult {
    branches: Vec<BranchRecord>,
}

impl BranchListResult {
    /// Creates a branch list result.
    #[must_use]
    pub fn new(branches: Vec<BranchRecord>) -> Self {
        Self { branches }
    }

    /// Returns the listed branches.
    #[must_use]
    pub fn branches(&self) -> &[BranchRecord] {
        &self.branches
    }

    /// Returns the number of branches in the result.
    #[must_use]
    pub fn len(&self) -> usize {
        self.branches.len()
    }

    /// Returns `true` when no branches were listed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.branches.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_name_rejects_blank_values() {
        let result = BranchName::try_from("   ");

        assert_eq!(
            result,
            Err(CoreError::EmptyValue {
                field: "branch_name"
            })
        );
    }

    #[test]
    fn branch_name_trims_input() {
        let branch = BranchName::try_from("  feature/import-stage  ").expect("valid branch");

        assert_eq!(branch.as_str(), "feature/import-stage");
    }

    #[test]
    fn branch_name_rejects_main_branch() {
        let result = BranchName::try_from("main");

        assert_eq!(
            result,
            Err(CoreError::ReservedValue {
                field: "branch_name",
                value: "main".to_string(),
            })
        );
    }

    #[test]
    fn branch_name_rejects_parent_segments() {
        let result = BranchName::try_from("feature/../unsafe");

        assert_eq!(
            result,
            Err(CoreError::InvalidCharacter {
                field: "branch_name",
                character: '.',
            })
        );
    }

    #[test]
    fn branch_name_rejects_invalid_characters() {
        let result = BranchName::try_from("feature/@invalid");

        assert_eq!(
            result,
            Err(CoreError::InvalidCharacter {
                field: "branch_name",
                character: '@',
            })
        );
    }

    #[test]
    fn branch_name_rejects_overlong_values() {
        let value = format!("feature-{}", "x".repeat(200));
        let result = BranchName::try_from(value);

        assert!(matches!(
            result,
            Err(CoreError::TooLong {
                field: "branch_name",
                ..
            })
        ));
    }
}
