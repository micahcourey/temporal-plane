//! Demonstrates branch-aware import staging with the `LanceDB` backend.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use arrow_array as _;
use arrow_schema as _;
use futures as _;
use lance as _;
use lance_index as _;
use lancedb as _;
use mnemix_core::{
    AdvancedStorageBackend, BranchName, Confidence, ImportStageRequest, Importance, MemoryId,
    MemoryKind, MemoryRecord, MemoryRepository, QueryLimit, RecallBackend, ScopeId, SearchQuery,
    TagName,
};
use mnemix_lancedb::LanceDbBackend;
use serde_json as _;
use tempfile as _;
use thiserror as _;
use tokio as _;

fn example_path(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_secs();
    std::env::temp_dir().join(format!("mnemix-{name}-{suffix}"))
}

fn build_memory(id: &str, scope: &str, title: &str, detail: &str) -> MemoryRecord {
    MemoryRecord::builder(
        MemoryId::try_from(id).expect("valid id"),
        ScopeId::try_from(scope).expect("valid scope"),
        MemoryKind::Decision,
    )
    .title(title)
    .expect("valid title")
    .summary(title)
    .expect("valid summary")
    .detail(detail)
    .expect("valid detail")
    .importance(Importance::new(90).expect("valid importance"))
    .confidence(Confidence::new(95).expect("valid confidence"))
    .add_tag(TagName::try_from("branch-experiment").expect("valid tag"))
    .metadata(BTreeMap::from([(
        "source".to_string(),
        "example".to_string(),
    )]))
    .build()
    .expect("memory should build")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_path = example_path("main");
    let source_path = example_path("source");
    std::fs::create_dir_all(&main_path)?;
    std::fs::create_dir_all(&source_path)?;

    let mut main_backend = LanceDbBackend::init(&main_path)?;
    let mut source_backend = LanceDbBackend::init(&source_path)?;

    main_backend.remember(build_memory(
        "memory:main",
        "repo:mnemix",
        "Main branch baseline",
        "This memory exists on the default branch.",
    ))?;
    source_backend.remember(build_memory(
        "memory:branch-only",
        "repo:mnemix",
        "Experiment branch change",
        "This memory should stage onto the experiment branch only.",
    ))?;

    let branch_name = BranchName::try_from("experiments/branch-import")?;
    let staged = main_backend.stage_import(
        &ImportStageRequest::new(&source_path).with_branch_name(branch_name.clone()),
    )?;

    println!("Staged import on branch: {}", staged.branch_name().as_str());
    println!("Staged records: {}", staged.staged_records());

    let branches = main_backend.list_branches()?;
    println!("Visible branches: {}", branches.len());
    for branch in branches.branches() {
        println!(
            "- {} (base v{})",
            branch.name().as_str(),
            branch.base_version().value()
        );
    }

    let main_results = main_backend.search(&SearchQuery::new(
        "branch",
        Some(ScopeId::try_from("repo:mnemix")?),
        QueryLimit::new(10)?,
    )?)?;
    println!("Main branch search results: {}", main_results.len());

    if main_backend.delete_branch(&branch_name).is_err() {
        println!("Branch deletion is blocked while staged changes remain.");
    }

    Ok(())
}
