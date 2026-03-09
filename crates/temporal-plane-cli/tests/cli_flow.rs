#![allow(deprecated, missing_docs, unused_crate_dependencies)]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("temporal-plane").expect("binary should build")
}

fn run_json_ok(store: &Path, args: &[&str]) -> Value {
    let assert = cli()
        .args(["--store", &store.display().to_string(), "--json"])
        .args(args)
        .assert()
        .success();

    serde_json::from_slice(&assert.get_output().stdout).expect("stdout should be valid json")
}

fn init_store(store: &Path) -> Value {
    run_json_ok(store, &["init"])
}

fn remember_demo_memory(store: &Path) -> Value {
    run_json_ok(
        store,
        &[
            "remember",
            "--id",
            "memory:cli-1",
            "--scope",
            "repo:temporal-plane",
            "--kind",
            "decision",
            "--title",
            "Freeze the CLI contract",
            "--summary",
            "Keep rendering separate from execution",
            "--detail",
            "This record drives the CLI MVP inspection flow.",
            "--importance",
            "90",
            "--confidence",
            "95",
            "--tag",
            "milestone-3",
            "--tag",
            "cli",
            "--entity",
            "TemporalPlane",
            "--pin-reason",
            "Used in CLI snapshots",
            "--metadata",
            "owner=cli",
            "--source-session-id",
            "session:cli",
            "--source-tool",
            "copilot",
            "--source-ref",
            "docs/temporal-plane-roadmap.md",
        ],
    )
}

fn create_checkpoint(store: &Path) -> Value {
    run_json_ok(
        store,
        &[
            "checkpoint",
            "--name",
            "milestone-3",
            "--description",
            "CLI MVP baseline",
        ],
    )
}

fn remember_restore_candidate(store: &Path) -> Value {
    run_json_ok(
        store,
        &[
            "remember",
            "--id",
            "memory:cli-2",
            "--scope",
            "repo:temporal-plane",
            "--kind",
            "decision",
            "--title",
            "Temporary restore candidate",
            "--summary",
            "Should disappear after restore",
            "--detail",
            "This memory only exists to validate restore semantics.",
        ],
    )
}

#[test]
fn init_and_full_inspection_flow_outputs_stable_json() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");

    let init = init_store(&store);
    assert_eq!(init["kind"], "status");
    assert_eq!(init["data"]["command"], "init");
    assert_eq!(init["data"]["schema_version"], 1);

    let remember = remember_demo_memory(&store);
    assert_eq!(remember["kind"], "memory");
    assert_eq!(remember["data"]["memory"]["id"], "memory:cli-1");
    assert_eq!(remember["data"]["memory"]["pinned"], true);

    let show = run_json_ok(&store, &["show", "--id", "memory:cli-1"]);
    assert_eq!(show["kind"], "memory");
    assert_eq!(show["data"]["command"], "show");
    assert_eq!(show["data"]["memory"]["source_tool"], "copilot");

    let recall = run_json_ok(
        &store,
        &[
            "recall",
            "--text",
            "contract",
            "--scope",
            "repo:temporal-plane",
        ],
    );
    assert_eq!(recall["kind"], "recall");
    assert_eq!(recall["data"]["count"], 1);
    assert_eq!(
        recall["data"]["pinned_context"][0]["memory"]["id"],
        "memory:cli-1"
    );
    assert_eq!(
        recall["data"]["pinned_context"][0]["layer"],
        "pinned_context"
    );
    assert_eq!(
        recall["data"]["archival"]
            .as_array()
            .expect("archival array")
            .len(),
        0
    );

    let search = run_json_ok(
        &store,
        &[
            "search",
            "--text",
            "contract",
            "--scope",
            "repo:temporal-plane",
        ],
    );
    assert_eq!(search["kind"], "memory_list");
    assert_eq!(search["data"]["count"], 1);
    assert_eq!(
        search["data"]["memories"][0]["title"],
        "Freeze the CLI contract"
    );

    let pins = run_json_ok(&store, &["pins", "--scope", "repo:temporal-plane"]);
    assert_eq!(pins["kind"], "memory_list");
    assert_eq!(pins["data"]["count"], 1);
    assert_eq!(
        pins["data"]["memories"][0]["pin_reason"],
        "Used in CLI snapshots"
    );

    let checkpoint = create_checkpoint(&store);
    assert_eq!(checkpoint["kind"], "checkpoint");
    assert_eq!(checkpoint["data"]["checkpoint"]["name"], "milestone-3");

    let history = run_json_ok(&store, &["history"]);
    assert_eq!(history["kind"], "version_list");
    assert!(history["data"]["count"].as_u64().expect("history count") >= 1);
    assert_eq!(
        history["data"]["versions"][0]["checkpoint_name"],
        "milestone-3"
    );

    let versions = run_json_ok(&store, &["versions"]);
    assert_eq!(versions["kind"], "version_list");
    assert!(versions["data"]["count"].as_u64().expect("version count") >= 1);

    let stats = run_json_ok(&store, &["stats", "--scope", "repo:temporal-plane"]);
    assert_eq!(stats["kind"], "stats");
    assert_eq!(stats["data"]["stats"]["total_memories"], 1);
    assert_eq!(stats["data"]["stats"]["pinned_memories"], 1);
    assert_eq!(stats["data"]["stats"]["latest_checkpoint"], "milestone-3");
}

#[test]
fn export_surfaces_success_as_json_status() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);
    let destination = temp_dir.path().join("export-store");

    let exported = cli()
        .args([
            "--store",
            &store.display().to_string(),
            "--json",
            "export",
            "--destination",
        ])
        .arg(&destination)
        .assert()
        .success();

    let output: Value =
        serde_json::from_slice(&exported.get_output().stdout).expect("stdout should be valid json");
    assert_eq!(output["kind"], "status");
    assert_eq!(output["data"]["command"], "export");
    assert_eq!(output["data"]["status"], "ok");
    assert_eq!(output["data"]["path"], store.display().to_string());
    assert!(
        output["data"]["message"]
            .as_str()
            .expect("string message")
            .contains(&destination.display().to_string())
    );
    assert!(destination.exists());
}

#[test]
fn show_surfaces_missing_memory_as_json_error() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);

    let assert = cli()
        .args([
            "--store",
            &store.display().to_string(),
            "--json",
            "show",
            "--id",
            "memory:missing",
        ])
        .assert()
        .failure();

    let error: Value =
        serde_json::from_slice(&assert.get_output().stderr).expect("stderr should be valid json");
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "memory_not_found");
    assert!(
        error["message"]
            .as_str()
            .expect("string error")
            .contains("memory:missing")
    );
}

#[test]
fn history_scope_surfaces_cli_json_error() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);

    let assert = cli()
        .args([
            "--store",
            &store.display().to_string(),
            "--json",
            "history",
            "--scope",
            "repo:temporal-plane",
        ])
        .assert()
        .failure();

    let error: Value =
        serde_json::from_slice(&assert.get_output().stderr).expect("stderr should be valid json");
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "scoped_history_not_supported");
    assert_eq!(error["message"], "scoped history is not implemented yet");
}

#[test]
fn restore_and_optimize_commands_expose_stable_json() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");

    let _ = init_store(&store);
    let _ = remember_demo_memory(&store);
    let checkpoint = create_checkpoint(&store);
    let _ = remember_restore_candidate(&store);

    let restore = run_json_ok(&store, &["restore", "--checkpoint", "milestone-3"]);
    assert_eq!(restore["kind"], "restore");
    assert_eq!(restore["data"]["target"]["kind"], "checkpoint");
    assert_eq!(restore["data"]["target"]["name"], "milestone-3");
    assert_eq!(
        restore["data"]["restored_version"],
        checkpoint["data"]["checkpoint"]["version"]
    );
    assert!(
        restore["data"]["current_version"]
            .as_u64()
            .expect("current version")
            > restore["data"]["previous_version"]
                .as_u64()
                .expect("previous version")
    );
    assert!(
        restore["data"]["pre_restore_checkpoint"]["name"]
            .as_str()
            .expect("checkpoint name")
            .starts_with("pre-restore-v")
    );

    let missing_assert = cli()
        .args([
            "--store",
            &store.display().to_string(),
            "--json",
            "show",
            "--id",
            "memory:cli-2",
        ])
        .assert()
        .failure();
    let missing_error: Value = serde_json::from_slice(&missing_assert.get_output().stderr)
        .expect("stderr should be valid json");
    assert_eq!(missing_error["code"], "memory_not_found");

    let optimize = run_json_ok(&store, &["optimize"]);
    assert_eq!(optimize["kind"], "optimize");
    assert!(optimize["data"]["compacted"].is_boolean());
    assert_eq!(optimize["data"]["prune_old_versions"], false);
    assert_eq!(optimize["data"]["retention"]["minimum_age_days"], 30);
    assert_eq!(
        optimize["data"]["retention"]["error_if_tagged_old_versions"],
        true
    );
    assert!(
        optimize["data"]["pre_optimize_checkpoint"]["name"]
            .as_str()
            .expect("checkpoint name")
            .starts_with("pre-optimize-v")
    );
}
