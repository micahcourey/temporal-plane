//! Integration coverage for the local `LanceDB` backend.

use arrow_array as _;
use arrow_schema as _;
use futures as _;
use lance as _;
use lance_index as _;
use lancedb as _;
use serde_json as _;
use std::collections::BTreeMap;

use tempfile::TempDir;
use temporal_plane_core::{
    CheckpointName, CheckpointRequest, CheckpointSelector, CleanupMode, Confidence, HistoryQuery,
    Importance, MemoryId, MemoryKind, MemoryRecord, OptimizeRequest, PreOperationCheckpointPolicy,
    QueryLimit, RestoreRequest, ScopeId, SearchQuery, StatsQuery, TagName, VersionNumber,
    traits::{
        CheckpointBackend, HistoryBackend, MemoryRepository, OptimizeBackend, RecallBackend,
        RestoreBackend, StatsBackend,
    },
};
use temporal_plane_lancedb::LanceDbBackend;
use thiserror as _;
use tokio as _;

fn build_memory(id: &str, scope: &str, title: &str, detail: &str) -> MemoryRecord {
    MemoryRecord::builder(
        MemoryId::try_from(id).expect("valid id"),
        ScopeId::try_from(scope).expect("valid scope"),
        MemoryKind::Decision,
    )
    .title(title)
    .expect("valid title")
    .summary("summary")
    .expect("valid summary")
    .detail(detail)
    .expect("valid detail")
    .importance(Importance::new(90).expect("valid importance"))
    .confidence(Confidence::new(95).expect("valid confidence"))
    .add_tag(TagName::try_from("integration").expect("valid tag"))
    .metadata(BTreeMap::from([(
        "layer".to_string(),
        "integration".to_string(),
    )]))
    .build()
    .expect("memory should build")
}

#[test]
fn persists_memories_across_reopen() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    {
        let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
        backend
            .remember(build_memory(
                "memory:persisted",
                "repo:temporal-plane",
                "Persist across reopen",
                "Milestone 2 backend should reopen existing data.",
            ))
            .expect("memory should store");
    }

    let reopened = LanceDbBackend::open(temp_dir.path()).expect("backend should reopen");
    let fetched = reopened
        .get(&MemoryId::try_from("memory:persisted").expect("valid id"))
        .expect("lookup should succeed");

    assert!(fetched.is_some());
}

#[test]
fn search_checkpoint_stats_and_history_are_available() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
    backend
        .remember(build_memory(
            "memory:searchable",
            "repo:temporal-plane",
            "Searchable memory",
            "LanceDB full text search should find this record.",
        ))
        .expect("memory should store");
    backend
        .checkpoint(&CheckpointRequest::new(
            CheckpointName::try_from("integration-freeze").expect("valid checkpoint name"),
            Some("checkpoint before inspection".to_string()),
        ))
        .expect("checkpoint should be created");

    let search_results = backend
        .search(
            &SearchQuery::new(
                "full text search",
                Some(ScopeId::try_from("repo:temporal-plane").expect("valid scope")),
                QueryLimit::new(10).expect("valid limit"),
            )
            .expect("search query should build"),
        )
        .expect("search should succeed");
    let checkpoints = backend.list_checkpoints().expect("checkpoints should list");
    let stats = backend
        .stats(&StatsQuery::new(None))
        .expect("stats should load");
    let history = backend
        .history(&HistoryQuery::new(
            None,
            QueryLimit::new(10).expect("valid limit"),
        ))
        .expect("history should load");

    assert_eq!(search_results.len(), 1);
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(stats.total_memories(), 1);
    assert!(stats.latest_checkpoint().is_some());
    assert!(!history.is_empty());
}

#[test]
fn restore_flow_recovers_checkpointed_state() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
    backend
        .remember(build_memory(
            "memory:baseline",
            "repo:temporal-plane",
            "Restore baseline",
            "This state should be restored.",
        ))
        .expect("baseline memory should store");
    let checkpoint = backend
        .checkpoint(&CheckpointRequest::new(
            CheckpointName::try_from("restore-baseline").expect("valid checkpoint name"),
            Some("checkpoint for restore integration".to_string()),
        ))
        .expect("checkpoint should be created");
    backend
        .remember(build_memory(
            "memory:temporary",
            "repo:temporal-plane",
            "Temporary state",
            "This should disappear after restore.",
        ))
        .expect("temporary memory should store");

    let result = backend
        .restore(&RestoreRequest::new(CheckpointSelector::Named(
            CheckpointName::try_from("restore-baseline").expect("valid checkpoint name"),
        )))
        .expect("restore should succeed");

    assert_eq!(result.restored_version(), checkpoint.version());
    assert!(result.current_version().value() > result.previous_version().value());
    assert!(
        backend
            .get(&MemoryId::try_from("memory:baseline").expect("valid id"))
            .expect("lookup should succeed")
            .is_some()
    );
    assert!(
        backend
            .get(&MemoryId::try_from("memory:temporary").expect("valid id"))
            .expect("lookup should succeed")
            .is_none()
    );
}

#[test]
fn restore_flow_recovers_checkpointed_state_by_raw_version() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
    backend
        .remember(build_memory(
            "memory:baseline-version",
            "repo:temporal-plane",
            "Restore baseline",
            "This state should be restored by raw version.",
        ))
        .expect("baseline memory should store");
    let checkpoint = backend
        .checkpoint(&CheckpointRequest::new(
            CheckpointName::try_from("restore-version-baseline").expect("valid checkpoint name"),
            Some("checkpoint for version restore integration".to_string()),
        ))
        .expect("checkpoint should be created");
    backend
        .remember(build_memory(
            "memory:temporary-version",
            "repo:temporal-plane",
            "Temporary state",
            "This should disappear after restore by version.",
        ))
        .expect("temporary memory should store");

    let result = backend
        .restore(&RestoreRequest::new(CheckpointSelector::Version(
            checkpoint.version(),
        )))
        .expect("restore by version should succeed");

    assert_eq!(result.restored_version(), checkpoint.version());
    assert!(
        backend
            .get(&MemoryId::try_from("memory:temporary-version").expect("valid id"))
            .expect("lookup should succeed")
            .is_none()
    );
}

#[test]
fn restore_respects_custom_pre_restore_checkpoint_policy() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
    backend
        .remember(build_memory(
            "memory:skip-policy",
            "repo:temporal-plane",
            "Restore baseline",
            "This state should restore without an auto-checkpoint.",
        ))
        .expect("baseline memory should store");
    backend
        .checkpoint(&CheckpointRequest::new(
            CheckpointName::try_from("skip-policy-baseline").expect("valid checkpoint name"),
            Some("checkpoint for policy restore integration".to_string()),
        ))
        .expect("checkpoint should be created");
    backend
        .remember(build_memory(
            "memory:skip-policy-temp",
            "repo:temporal-plane",
            "Temporary state",
            "This should disappear after restore.",
        ))
        .expect("temporary memory should store");

    let request = RestoreRequest::new(CheckpointSelector::Named(
        CheckpointName::try_from("skip-policy-baseline").expect("valid checkpoint name"),
    ))
    .with_retention_policy(
        temporal_plane_core::RetentionPolicy::conservative()
            .with_pre_restore_checkpoint(PreOperationCheckpointPolicy::Skip),
    );
    let result = backend.restore(&request).expect("restore should succeed");

    assert!(result.pre_restore_checkpoint().is_none());
}

#[test]
fn restore_surfaces_unknown_checkpoint_and_version_errors() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");

    let checkpoint_error = backend
        .restore(&RestoreRequest::new(CheckpointSelector::Named(
            CheckpointName::try_from("missing-checkpoint").expect("valid checkpoint name"),
        )))
        .expect_err("missing checkpoint should fail");
    assert!(checkpoint_error.to_string().contains("missing-checkpoint"));

    let version_error = backend
        .restore(&RestoreRequest::new(CheckpointSelector::Version(
            VersionNumber::new(999),
        )))
        .expect_err("missing version should fail");
    assert!(version_error.to_string().contains("999"));
}

#[test]
fn optimize_prune_protects_tagged_versions() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
    backend
        .remember(build_memory(
            "memory:protected",
            "repo:temporal-plane",
            "Protected version",
            "Tagged versions must remain protected from routine pruning.",
        ))
        .expect("memory should store");
    backend
        .checkpoint(&CheckpointRequest::new(
            CheckpointName::try_from("protected-version").expect("valid checkpoint name"),
            Some("checkpoint before cleanup".to_string()),
        ))
        .expect("checkpoint should be created");
    backend
        .remember(build_memory(
            "memory:newer",
            "repo:temporal-plane",
            "Newer version",
            "Creates a later head for prune validation.",
        ))
        .expect("newer memory should store");

    let request = OptimizeRequest::new(
        temporal_plane_core::RetentionPolicy::conservative()
            .with_cleanup_mode(CleanupMode::AllowPrune)
            .with_minimum_age_days(0),
    )
    .with_prune_old_versions(true);
    let error = backend
        .optimize(&request)
        .expect_err("tagged versions should block pruning");

    assert!(error.to_string().to_lowercase().contains("tag"));
}

#[test]
fn optimize_prune_removes_old_untagged_versions() {
    let temp_dir = TempDir::new().expect("tempdir should be created");
    let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
    backend
        .remember(build_memory(
            "memory:prune-integration-baseline",
            "repo:temporal-plane",
            "Prunable version",
            "This creates an older untagged version.",
        ))
        .expect("baseline memory should store");
    backend
        .remember(build_memory(
            "memory:prune-integration-current",
            "repo:temporal-plane",
            "Current version",
            "This keeps a newer head after pruning.",
        ))
        .expect("current memory should store");

    let request = OptimizeRequest::new(
        temporal_plane_core::RetentionPolicy::conservative()
            .with_cleanup_mode(CleanupMode::AllowPrune)
            .with_minimum_age_days(0)
            .with_delete_unverified(true)
            .with_pre_optimize_checkpoint(PreOperationCheckpointPolicy::Skip),
    )
    .with_prune_old_versions(true);
    let result = backend
        .optimize(&request)
        .expect("untagged versions should be pruned");

    assert!(result.pruned_versions() > 0);
}
