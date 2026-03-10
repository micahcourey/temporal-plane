//! Typed identifiers and small value objects used throughout the product model.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::CoreError;

const DEFAULT_MAX_LEN: usize = 128;
const LONG_TEXT_MAX_LEN: usize = 256;

fn validate_text(field: &'static str, value: &str, max: usize) -> Result<(), CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyValue { field });
    }

    let actual = trimmed.chars().count();
    if actual > max {
        return Err(CoreError::TooLong { field, max, actual });
    }

    if let Some(character) = trimmed.chars().find(|character| character.is_control()) {
        return Err(CoreError::InvalidCharacter { field, character });
    }

    Ok(())
}

macro_rules! string_newtype {
    ($name:ident, $field:literal, $max:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated value from an owned string.
            ///
            /// # Errors
            ///
            /// Returns [`CoreError`] when the value is blank, too long, or
            /// contains control characters.
            pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
                let value = value.into();
                validate_text($field, &value, $max)?;
                Ok(Self(value.trim().to_owned()))
            }

            /// Returns the validated string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = CoreError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<String> for $name {
            type Error = CoreError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

string_newtype!(
    MemoryId,
    "memory_id",
    DEFAULT_MAX_LEN,
    "A stable identifier for a stored memory."
);
string_newtype!(
    ScopeId,
    "scope_id",
    DEFAULT_MAX_LEN,
    "A typed scope boundary used for grouping and retrieval."
);
string_newtype!(
    SessionId,
    "session_id",
    DEFAULT_MAX_LEN,
    "A source session identifier attached to a memory."
);
string_newtype!(
    ToolName,
    "tool_name",
    DEFAULT_MAX_LEN,
    "The logical tool name that produced or imported a memory."
);
string_newtype!(
    SourceRef,
    "source_ref",
    LONG_TEXT_MAX_LEN,
    "An external reference pointing back to the source of a memory."
);
string_newtype!(
    TagName,
    "tag_name",
    DEFAULT_MAX_LEN,
    "A retrieval tag applied to a memory record."
);
string_newtype!(
    EntityName,
    "entity_name",
    DEFAULT_MAX_LEN,
    "An entity label extracted into the durable memory model."
);
string_newtype!(
    CheckpointName,
    "checkpoint_name",
    DEFAULT_MAX_LEN,
    "A user-meaningful name for a stable checkpoint."
);

/// A typed timestamp wrapper for product-level recorded events.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct RecordedAt(SystemTime);

impl RecordedAt {
    /// Captures the current system time.
    #[must_use]
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    /// Wraps an existing system time.
    #[must_use]
    pub const fn new(value: SystemTime) -> Self {
        Self(value)
    }

    /// Returns the underlying system time.
    #[must_use]
    pub const fn value(self) -> SystemTime {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_id_rejects_blank_values() {
        let result = ScopeId::try_from("   ");

        assert_eq!(result, Err(CoreError::EmptyValue { field: "scope_id" }));
    }

    #[test]
    fn checkpoint_name_trims_input() {
        let checkpoint =
            CheckpointName::try_from("  milestone-1  ").expect("checkpoint should be valid");

        assert_eq!(checkpoint.as_str(), "milestone-1");
    }

    #[test]
    fn entity_name_rejects_control_characters() {
        let result = EntityName::try_from("repo\nname");

        assert_eq!(
            result,
            Err(CoreError::InvalidCharacter {
                field: "entity_name",
                character: '\n',
            })
        );
    }
}
