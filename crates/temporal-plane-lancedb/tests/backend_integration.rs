//! Integration coverage for the local `LanceDB` backend.

use arrow_array as _;
use arrow_schema as _;
use futures as _;
use lance_index as _;
use lancedb as _;
use serde_json as _;
use std::collections::BTreeMap;

use tempfile::TempDir;
use temporal_plane_core::{
    CheckpointName, CheckpointRequest, Confidence, HistoryQuery, Importance, MemoryId, MemoryKind,
    MemoryRecord, QueryLimit, ScopeId, SearchQuery, StatsQuery, TagName,
    traits::{CheckpointBackend, HistoryBackend, MemoryRepository, RecallBackend, StatsBackend},
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
