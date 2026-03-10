//! Shared value objects and contracts for Mnemix.

/// A minimal placeholder type used to validate the workspace scaffold.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlaceholderType {
    name: String,
}

impl PlaceholderType {
    /// Creates a new placeholder value.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the placeholder name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
