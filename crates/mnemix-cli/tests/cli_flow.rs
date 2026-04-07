#![allow(deprecated, missing_docs, unused_crate_dependencies)]

use std::{
    fs,
    io::ErrorKind,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    path::Path,
    thread,
    time::Duration,
};

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

const MOCK_SERVER_IDLE_POLL_INTERVAL: Duration = Duration::from_millis(25);
const MOCK_SERVER_MAX_IDLE_POLLS: u16 = 600;

fn cli() -> Command {
    Command::cargo_bin("mnemix").expect("binary should build")
}

fn cli_alias() -> Command {
    Command::cargo_bin("mx").expect("alias binary should build")
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

fn run_json_ok_with_config_home(config_home: &Path, args: &[&str]) -> Value {
    let assert = cli()
        .env("MNEMIX_CONFIG_HOME", config_home)
        .args(["--json"])
        .args(args)
        .assert()
        .success();

    serde_json::from_slice(&assert.get_output().stdout).expect("stdout should be valid json")
}

fn write_policy_config(store: &Path) {
    fs::write(
        store.join("policy.toml"),
        r#"
version = 1

[defaults]
scope_strategy = "repo"
evidence_ttl = "task"

[[rules]]
id = "commit-writeback"
trigger = "on_git_commit"
mode = "required_with_skip_reason"
requires = ["writeback"]
allow_skip = true
on_unsatisfied = "block"

[rules.when]
host = ["coding-agent"]
paths_any = ["adapters/**"]
exclude_paths = ["docs/**"]
"#,
    )
    .expect("policy config should be written");
}

fn vectors_show(store: &Path) -> Value {
    run_json_ok(store, &["vectors", "show"])
}

fn vectors_enable(store: &Path) -> Value {
    run_json_ok(
        store,
        &[
            "vectors",
            "enable",
            "--model",
            "test-embedder",
            "--dimensions",
            "3",
        ],
    )
}

fn vectors_enable_with_provider(store: &Path, config_home: &Path, provider: &str) -> Value {
    run_json_ok_with_config_home(
        config_home,
        &[
            "--store",
            &store.display().to_string(),
            "vectors",
            "enable",
            "--provider",
            provider,
            "--auto-embed-on-write",
        ],
    )
}

fn vectors_backfill(store: &Path) -> Value {
    run_json_ok(store, &["vectors", "backfill"])
}

fn vectors_backfill_apply_error(store: &Path, config_home: &Path) -> Value {
    let assert = cli()
        .env("MNEMIX_CONFIG_HOME", config_home)
        .args(["--store", &store.display().to_string(), "--json"])
        .args(["vectors", "backfill", "--apply"])
        .assert()
        .failure();

    serde_json::from_slice(&assert.get_output().stderr).expect("stderr should be valid json")
}

fn providers_show_error(config_home: &Path, name: &str) -> Value {
    let assert = cli()
        .env("MNEMIX_CONFIG_HOME", config_home)
        .args(["--json", "providers", "show", "--name", name])
        .assert()
        .failure();

    serde_json::from_slice(&assert.get_output().stderr).expect("stderr should be valid json")
}

fn providers_validate_error(config_home: &Path, name: &str) -> Value {
    let assert = cli()
        .env("MNEMIX_CONFIG_HOME", config_home)
        .args(["--json", "providers", "validate", "--name", name])
        .assert()
        .failure();

    serde_json::from_slice(&assert.get_output().stderr).expect("stderr should be valid json")
}

fn run_json_error(store: &Path, args: &[&str]) -> Value {
    let assert = cli()
        .args(["--store", &store.display().to_string(), "--json"])
        .args(args)
        .assert()
        .failure();

    serde_json::from_slice(&assert.get_output().stderr).expect("stderr should be valid json")
}

fn start_mock_embeddings_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    listener
        .set_nonblocking(true)
        .expect("mock server should support nonblocking accept");
    let address = listener.local_addr().expect("local addr");
    let handle = thread::spawn(move || {
        let mut idle_rounds = 0_u16;
        loop {
            let mut stream = match listener.accept() {
                Ok((stream, _)) => {
                    idle_rounds = 0;
                    stream
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    idle_rounds += 1;
                    if idle_rounds >= MOCK_SERVER_MAX_IDLE_POLLS {
                        break;
                    }
                    thread::sleep(MOCK_SERVER_IDLE_POLL_INTERVAL);
                    continue;
                }
                Err(error) => panic!("incoming stream should be accepted: {error}"),
            };
            stream
                .set_nonblocking(false)
                .expect("accepted stream should switch back to blocking mode");
            let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
            let mut request_line = String::new();
            reader
                .read_line(&mut request_line)
                .expect("request line should read");
            let mut content_length = 0_usize;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).expect("header should read");
                if header == "\r\n" {
                    break;
                }
                let lower = header.to_ascii_lowercase();
                if let Some(value) = lower.strip_prefix("content-length:") {
                    content_length = value.trim().parse().expect("content-length should parse");
                }
            }

            let mut body = vec![0_u8; content_length];
            reader.read_exact(&mut body).expect("body should read");
            let payload: Value = serde_json::from_slice(&body).expect("json body");
            let model = payload["model"].as_str().expect("model string");
            let response = serde_json::json!({
                "object": "list",
                "model": model,
                "data": [
                    {
                        "object": "embedding",
                        "index": 0,
                        "embedding": [0.11_f32, 0.22_f32, 0.33_f32]
                    }
                ]
            });
            let response_bytes =
                serde_json::to_vec(&response).expect("mock response should serialize");
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response_bytes.len()
            )
            .expect("response headers should write");
            stream
                .write_all(&response_bytes)
                .expect("response body should write");
            stream.flush().expect("response should flush");
        }
    });

    (format!("http://{address}/v1"), handle)
}

fn remember_demo_memory(store: &Path) -> Value {
    run_json_ok(
        store,
        &[
            "remember",
            "--id",
            "memory:cli-1",
            "--scope",
            "repo:mnemix",
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
            "Mnemix",
            "--pin-reason",
            "Used in CLI snapshots",
            "--metadata",
            "owner=cli",
            "--source-session-id",
            "session:cli",
            "--source-tool",
            "copilot",
            "--source-ref",
            "docs/mnemix-roadmap.md",
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

fn export_store_with_config_home(store: &Path, config_home: &Path, destination: &Path) -> Value {
    let assert = cli()
        .env("MNEMIX_CONFIG_HOME", config_home)
        .args([
            "--store",
            &store.display().to_string(),
            "--json",
            "export",
            "--destination",
        ])
        .arg(destination)
        .assert()
        .success();

    serde_json::from_slice(&assert.get_output().stdout).expect("stdout should be valid json")
}

fn tree_contains_bytes(path: &Path, needle: &[u8]) -> bool {
    if path.is_file() {
        return fs::read(path)
            .map(|contents| {
                contents
                    .windows(needle.len())
                    .any(|window| window == needle)
            })
            .unwrap_or(false);
    }

    if !path.is_dir() {
        return false;
    }

    fs::read_dir(path)
        .expect("directory should be readable")
        .filter_map(Result::ok)
        .any(|entry| tree_contains_bytes(&entry.path(), needle))
}

fn closed_endpoint() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("temporary listener should bind");
    let address = listener.local_addr().expect("local addr");
    drop(listener);
    format!("http://{address}/v1")
}

fn remember_restore_candidate(store: &Path) -> Value {
    run_json_ok(
        store,
        &[
            "remember",
            "--id",
            "memory:cli-2",
            "--scope",
            "repo:mnemix",
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
    assert_eq!(init["data"]["schema_version"], 4);

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
        &["recall", "--text", "contract", "--scope", "repo:mnemix"],
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
        &["search", "--text", "contract", "--scope", "repo:mnemix"],
    );
    assert_eq!(search["kind"], "memory_list");
    assert_eq!(search["data"]["count"], 1);
    assert_eq!(
        search["data"]["memories"][0]["title"],
        "Freeze the CLI contract"
    );

    let pins = run_json_ok(&store, &["pins", "--scope", "repo:mnemix"]);
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

    let stats = run_json_ok(&store, &["stats", "--scope", "repo:mnemix"]);
    assert_eq!(stats["kind"], "stats");
    assert_eq!(stats["data"]["stats"]["total_memories"], 1);
    assert_eq!(stats["data"]["stats"]["pinned_memories"], 1);
    assert_eq!(stats["data"]["stats"]["latest_checkpoint"], "milestone-3");
}

#[test]
fn vector_commands_surface_stable_json_status() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");

    let _ = init_store(&store);

    let show = vectors_show(&store);
    assert_eq!(show["kind"], "status");
    assert_eq!(show["data"]["command"], "vectors show");
    assert!(
        show["data"]["message"]
            .as_str()
            .expect("string message")
            .contains("vectors_enabled=false")
    );
    assert!(
        show["data"]["message"]
            .as_str()
            .expect("string message")
            .contains("indexable_embedding_storage=false")
    );
    assert!(
        show["data"]["message"]
            .as_str()
            .expect("string message")
            .contains("embedding_coverage_percent=0")
    );
    assert!(
        show["data"]["message"]
            .as_str()
            .expect("string message")
            .contains("vector_index_available=false")
    );

    let enable = vectors_enable(&store);
    assert_eq!(enable["kind"], "status");
    assert_eq!(enable["data"]["command"], "vectors enable");
    assert!(
        enable["data"]["message"]
            .as_str()
            .expect("string message")
            .contains("model=test-embedder")
    );

    let _ = remember_demo_memory(&store);
    let show_after_write = vectors_show(&store);
    assert!(
        show_after_write["data"]["message"]
            .as_str()
            .expect("string message")
            .contains("embedded_memories=0/1")
    );
    let backfill = vectors_backfill(&store);
    assert_eq!(backfill["kind"], "status");
    assert_eq!(backfill["data"]["command"], "vectors backfill");
    assert!(
        backfill["data"]["message"]
            .as_str()
            .expect("string message")
            .contains("candidate_memories=1")
    );
}

#[test]
fn vector_backfill_apply_requires_provider_argument() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let config_home = temp_dir.path().join("config-home");

    let _ = init_store(&store);
    let error = vectors_backfill_apply_error(&store, &config_home);
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_required");
}

#[test]
fn provider_profile_commands_surface_stable_json() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");

    let empty_list = run_json_ok_with_config_home(&config_home, &["providers", "list"]);
    assert_eq!(empty_list["kind"], "provider_profile_list");
    assert_eq!(empty_list["data"]["count"], 0);
    assert!(
        empty_list["data"]["config_path"]
            .as_str()
            .expect("config path")
            .ends_with("mnemix/providers.toml")
    );

    let set_cloud = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-cloud",
            "--name",
            "openai",
            "--model",
            "text-embedding-3-small",
            "--base-url",
            "https://api.openai.com/v1",
            "--api-key-env",
            "OPENAI_API_KEY",
        ],
    );
    assert_eq!(set_cloud["kind"], "status");
    assert_eq!(set_cloud["data"]["command"], "providers set-cloud");

    let set_local = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-local",
            "--name",
            "ollama",
            "--model",
            "nomic-embed-text",
            "--endpoint",
            "http://127.0.0.1:11434/v1",
        ],
    );
    assert_eq!(set_local["kind"], "status");
    assert_eq!(set_local["data"]["command"], "providers set-local");

    let show_cloud =
        run_json_ok_with_config_home(&config_home, &["providers", "show", "--name", "openai"]);
    assert_eq!(show_cloud["kind"], "provider_profile");
    assert_eq!(show_cloud["data"]["profile"]["kind"], "cloud");
    assert_eq!(
        show_cloud["data"]["profile"]["api_key_source"],
        "env:OPENAI_API_KEY"
    );

    let list_profiles = run_json_ok_with_config_home(&config_home, &["providers", "list"]);
    assert_eq!(list_profiles["kind"], "provider_profile_list");
    assert_eq!(list_profiles["data"]["count"], 2);
    assert_eq!(list_profiles["data"]["profiles"][0]["name"], "ollama");
    assert_eq!(list_profiles["data"]["profiles"][1]["name"], "openai");

    let config_path = config_home.join("mnemix").join("providers.toml");
    let payload = fs::read_to_string(&config_path).expect("provider config should be written");
    assert!(payload.contains("OPENAI_API_KEY"));
    assert!(!payload.contains("sk-secret-value"));

    let removed =
        run_json_ok_with_config_home(&config_home, &["providers", "remove", "--name", "openai"]);
    assert_eq!(removed["kind"], "status");
    assert_eq!(removed["data"]["command"], "providers remove");

    let remaining = run_json_ok_with_config_home(&config_home, &["providers", "list"]);
    assert_eq!(remaining["data"]["count"], 1);
    assert_eq!(remaining["data"]["profiles"][0]["name"], "ollama");
}

#[test]
fn provider_validate_and_backfill_apply_work_with_local_profile() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");
    let store = temp_dir.path().join("store");
    let (endpoint, server) = start_mock_embeddings_server();

    let _ = init_store(&store);
    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-local",
            "--name",
            "mock-local",
            "--model",
            "test-embedder",
            "--endpoint",
            &endpoint,
        ],
    );
    let _ = vectors_enable(&store);
    let _ = remember_demo_memory(&store);

    let validated = run_json_ok_with_config_home(
        &config_home,
        &["providers", "validate", "--name", "mock-local"],
    );
    assert_eq!(validated["kind"], "status");
    assert_eq!(validated["data"]["command"], "providers validate");
    assert!(
        validated["data"]["message"]
            .as_str()
            .expect("message")
            .contains("dimensions=3")
    );

    let show = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "vectors",
            "show",
            "--provider",
            "mock-local",
        ],
    );
    assert_eq!(show["kind"], "status");
    assert!(
        show["data"]["message"]
            .as_str()
            .expect("message")
            .contains("has_provider=true")
    );

    let backfill = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "vectors",
            "backfill",
            "--apply",
            "--provider",
            "mock-local",
        ],
    );
    assert_eq!(backfill["kind"], "status");
    assert!(
        backfill["data"]["message"]
            .as_str()
            .expect("message")
            .contains("updated_memories=1")
    );

    server.join().expect("mock server should finish");
}

#[test]
fn provider_validate_reports_store_compatibility_and_vectors_enable_can_use_provider() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");
    let store = temp_dir.path().join("store");
    let (endpoint, server) = start_mock_embeddings_server();

    let _ = init_store(&store);
    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-local",
            "--name",
            "mock-local",
            "--model",
            "test-embedder",
            "--endpoint",
            &endpoint,
        ],
    );

    let validated_before_enable = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "providers",
            "validate",
            "--name",
            "mock-local",
        ],
    );
    assert_eq!(validated_before_enable["kind"], "status");
    assert_eq!(validated_before_enable["data"]["status"], "ok");
    assert!(
        validated_before_enable["data"]["message"]
            .as_str()
            .expect("message")
            .contains("store_compatibility=vectors_disabled")
    );

    let enabled = vectors_enable_with_provider(&store, &config_home, "mock-local");
    assert_eq!(enabled["kind"], "status");
    assert_eq!(enabled["data"]["command"], "vectors enable");
    assert!(
        enabled["data"]["message"]
            .as_str()
            .expect("message")
            .contains("provider=mock-local")
    );
    assert!(
        enabled["data"]["message"]
            .as_str()
            .expect("message")
            .contains("model=test-embedder")
    );

    let validated_after_enable = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "providers",
            "validate",
            "--name",
            "mock-local",
        ],
    );
    assert_eq!(validated_after_enable["kind"], "status");
    assert_eq!(validated_after_enable["data"]["status"], "ok");
    assert!(
        validated_after_enable["data"]["message"]
            .as_str()
            .expect("message")
            .contains("store_compatibility=matched")
    );

    let show = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "vectors",
            "show",
            "--provider",
            "mock-local",
        ],
    );
    assert_eq!(show["kind"], "status");
    assert_eq!(show["data"]["status"], "ok");
    assert!(
        show["data"]["message"]
            .as_str()
            .expect("message")
            .contains("provider_compatibility=matched")
    );

    server.join().expect("mock server should finish");
}

#[test]
fn provider_store_mismatch_is_reported_cleanly() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");
    let store = temp_dir.path().join("store");
    let (endpoint, server) = start_mock_embeddings_server();

    let _ = init_store(&store);
    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-local",
            "--name",
            "mock-local",
            "--model",
            "test-embedder",
            "--endpoint",
            &endpoint,
        ],
    );
    let _ = run_json_ok(
        &store,
        &[
            "vectors",
            "enable",
            "--model",
            "different-model",
            "--dimensions",
            "3",
        ],
    );

    let validated = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "providers",
            "validate",
            "--name",
            "mock-local",
        ],
    );
    assert_eq!(validated["kind"], "status");
    assert_eq!(validated["data"]["status"], "mismatch");
    assert!(
        validated["data"]["message"]
            .as_str()
            .expect("message")
            .contains("store_compatibility=model_mismatch")
    );

    let show = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "vectors",
            "show",
            "--provider",
            "mock-local",
        ],
    );
    assert_eq!(show["kind"], "status");
    assert_eq!(show["data"]["status"], "mismatch");
    assert!(
        show["data"]["message"]
            .as_str()
            .expect("message")
            .contains("provider_compatibility=model_mismatch")
    );

    let error = {
        let assert = cli()
            .env("MNEMIX_CONFIG_HOME", &config_home)
            .args(["--store", &store.display().to_string(), "--json"])
            .args([
                "search",
                "--text",
                "contract",
                "--scope",
                "repo:mnemix",
                "--mode",
                "semantic",
                "--provider",
                "mock-local",
            ])
            .assert()
            .failure();
        serde_json::from_slice::<Value>(&assert.get_output().stderr).expect("stderr json")
    };
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_store_incompatible");
    assert!(
        error["message"]
            .as_str()
            .expect("error message")
            .contains("different-model")
    );

    server.join().expect("mock server should finish");
}

#[test]
fn cloud_provider_missing_secret_reports_actionable_error() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");

    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-cloud",
            "--name",
            "openai",
            "--model",
            "text-embedding-3-small",
            "--base-url",
            "https://api.openai.com/v1",
            "--api-key-env",
            "OPENAI_API_KEY",
        ],
    );

    let error = providers_validate_error(&config_home, "openai");
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_secret_missing");
    assert!(
        error["message"]
            .as_str()
            .expect("error message")
            .contains("OPENAI_API_KEY")
    );
}

#[test]
fn local_provider_unreachable_runtime_reports_actionable_error() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");
    let endpoint = closed_endpoint();

    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-local",
            "--name",
            "offline-local",
            "--model",
            "test-embedder",
            "--endpoint",
            &endpoint,
        ],
    );

    let error = providers_validate_error(&config_home, "offline-local");
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_runtime_error");
    assert!(
        error["message"]
            .as_str()
            .expect("error message")
            .contains("offline-local")
    );
}

#[test]
fn provider_config_does_not_leak_into_store_export() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");
    let store = temp_dir.path().join("store");
    let destination = temp_dir.path().join("export-store");

    let _ = init_store(&store);
    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-cloud",
            "--name",
            "openai",
            "--model",
            "text-embedding-3-small",
            "--base-url",
            "https://api.openai.com/v1",
            "--api-key-env",
            "OPENAI_API_KEY",
        ],
    );
    let exported = export_store_with_config_home(&store, &config_home, &destination);
    assert_eq!(exported["kind"], "status");
    assert!(destination.exists());
    assert!(!tree_contains_bytes(&destination, b"OPENAI_API_KEY"));
    assert!(!tree_contains_bytes(
        &destination,
        b"text-embedding-3-small"
    ));
}

#[test]
fn semantic_recall_requires_provider_argument() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");

    let _ = init_store(&store);
    let error = run_json_error(
        &store,
        &["recall", "--text", "contract", "--mode", "semantic"],
    );
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_required");
}

#[test]
fn hybrid_search_requires_provider_argument() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");

    let _ = init_store(&store);
    let error = run_json_error(
        &store,
        &["search", "--text", "contract", "--mode", "hybrid"],
    );
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_required");
}

#[test]
fn semantic_recall_and_hybrid_search_surface_provider_and_provenance() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");
    let store = temp_dir.path().join("store");
    let (endpoint, server) = start_mock_embeddings_server();

    let _ = init_store(&store);
    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "providers",
            "set-local",
            "--name",
            "mock-local",
            "--model",
            "test-embedder",
            "--endpoint",
            &endpoint,
        ],
    );
    let _ = vectors_enable(&store);
    let _ = remember_demo_memory(&store);
    let _ = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "vectors",
            "backfill",
            "--apply",
            "--provider",
            "mock-local",
        ],
    );

    let recall = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "recall",
            "--text",
            "style guide",
            "--scope",
            "repo:mnemix",
            "--mode",
            "semantic",
            "--provider",
            "mock-local",
        ],
    );
    assert_eq!(recall["kind"], "recall");
    assert_eq!(recall["data"]["retrieval_mode"], "semantic");
    assert_eq!(recall["data"]["provider"], "mock-local");
    let pinned_context = recall["data"]["pinned_context"]
        .as_array()
        .expect("pinned context array");
    assert_eq!(pinned_context.len(), 1);
    assert!(
        pinned_context[0]["reasons"]
            .as_array()
            .expect("reasons array")
            .iter()
            .any(|value| value == "semantic_match")
    );

    let search = run_json_ok_with_config_home(
        &config_home,
        &[
            "--store",
            &store.display().to_string(),
            "search",
            "--text",
            "contract",
            "--scope",
            "repo:mnemix",
            "--mode",
            "hybrid",
            "--provider",
            "mock-local",
        ],
    );
    assert_eq!(search["kind"], "memory_list");
    assert_eq!(search["data"]["retrieval_mode"], "hybrid");
    assert_eq!(search["data"]["provider"], "mock-local");
    assert_eq!(
        search["data"]["memories"][0]["search_match"]["kind"],
        "hybrid"
    );
    assert_eq!(
        search["data"]["memories"][0]["search_match"]["lexical"],
        true
    );
    assert_eq!(
        search["data"]["memories"][0]["search_match"]["semantic"],
        true
    );
    assert!(
        search["data"]["memories"][0]["search_match"]["semantic_score"]
            .as_str()
            .is_some()
    );

    server.join().expect("mock server should finish");
}

#[test]
fn providers_show_missing_profile_reports_error() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");

    let error = providers_show_error(&config_home, "missing");
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_profile_not_found");
}

#[test]
fn providers_validate_missing_profile_reports_error() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let config_home = temp_dir.path().join("config-home");

    let error = providers_validate_error(&config_home, "missing");
    assert_eq!(error["kind"], "error");
    assert_eq!(error["code"], "provider_profile_not_found");
}

#[test]
fn mx_alias_binary_supports_help() {
    cli_alias().arg("--help").assert().success();
}

#[test]
fn ui_command_supports_help() {
    cli().args(["ui", "--help"]).assert().success();
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
fn policy_check_without_config_defaults_to_allow() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");

    let output = run_json_ok(
        &store,
        &[
            "policy",
            "check",
            "--trigger",
            "on_git_commit",
            "--host",
            "coding-agent",
        ],
    );

    assert_eq!(output["kind"], "policy");
    assert_eq!(output["data"]["action"], "check");
    assert_eq!(output["data"]["decision"], "allow");
    assert_eq!(
        output["data"]["reasons"][0],
        "No `policy.toml` file was found or it did not contain any rules; defaulting to allow."
    );
}

#[test]
fn policy_recorded_writeback_satisfies_commit_check() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);
    write_policy_config(&store);

    let blocked = run_json_ok(
        &store,
        &[
            "policy",
            "check",
            "--trigger",
            "on_git_commit",
            "--workflow-key",
            "commit-1",
            "--host",
            "coding-agent",
            "--path",
            "adapters/coding_agent_adapter.py",
        ],
    );
    assert_eq!(blocked["kind"], "policy");
    assert_eq!(blocked["data"]["decision"], "block");
    assert_eq!(blocked["data"]["missing_actions"][0], "writeback");

    let recorded = run_json_ok(
        &store,
        &[
            "policy",
            "record",
            "--workflow-key",
            "commit-1",
            "--action",
            "writeback",
        ],
    );
    assert_eq!(recorded["kind"], "status");
    assert_eq!(recorded["data"]["status"], "recorded");

    let satisfied = run_json_ok(
        &store,
        &[
            "policy",
            "explain",
            "--trigger",
            "on_git_commit",
            "--workflow-key",
            "commit-1",
            "--host",
            "coding-agent",
            "--path",
            "adapters/coding_agent_adapter.py",
        ],
    );
    assert_eq!(satisfied["kind"], "policy");
    assert_eq!(satisfied["data"]["action"], "explain");
    assert_eq!(satisfied["data"]["decision"], "allow");
    assert_eq!(satisfied["data"]["matched_rules"][0]["satisfied"], true);
}

#[test]
fn policy_explain_preserves_legacy_state_entries() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);
    write_policy_config(&store);

    fs::write(
        store.join("policy-state.json"),
        r#"{
  "workflows": {
    "commit-1": {
      "actions": ["writeback"],
      "skip_reason": null
    }
  }
}"#,
    )
    .expect("legacy policy state should be written");

    let explained = run_json_ok(
        &store,
        &[
            "policy",
            "explain",
            "--trigger",
            "on_git_commit",
            "--workflow-key",
            "commit-1",
            "--host",
            "coding-agent",
            "--path",
            "adapters/coding_agent_adapter.py",
        ],
    );
    assert_eq!(explained["data"]["decision"], "allow");
    assert_eq!(explained["data"]["matched_rules"][0]["satisfied"], true);
}

#[test]
fn policy_clear_removes_recorded_action_from_workflow() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);
    write_policy_config(&store);

    let _ = run_json_ok(
        &store,
        &[
            "policy",
            "record",
            "--workflow-key",
            "commit-1",
            "--action",
            "writeback",
        ],
    );

    let cleared = run_json_ok(
        &store,
        &[
            "policy",
            "clear",
            "--workflow-key",
            "commit-1",
            "--action",
            "writeback",
        ],
    );
    assert_eq!(cleared["kind"], "status");
    assert_eq!(cleared["data"]["status"], "cleared");

    let blocked = run_json_ok(
        &store,
        &[
            "policy",
            "explain",
            "--trigger",
            "on_git_commit",
            "--workflow-key",
            "commit-1",
            "--host",
            "coding-agent",
            "--path",
            "adapters/coding_agent_adapter.py",
        ],
    );
    assert_eq!(blocked["data"]["decision"], "block");
    assert_eq!(blocked["data"]["missing_actions"][0], "writeback");
}

#[test]
fn policy_clear_noop_does_not_refresh_unrelated_evidence() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);
    write_policy_config(&store);

    let original = r#"{
  "workflows": {
    "commit-1": {
      "evidence": {
        "actions": ["recall"],
        "skip_reason": null
      },
      "evidence_ttl": "task",
      "created_at_unix": 10,
      "updated_at_unix": 20
    }
  }
}"#;
    fs::write(store.join("policy-state.json"), original).expect("policy state should be written");

    let cleared = run_json_ok(
        &store,
        &[
            "policy",
            "clear",
            "--workflow-key",
            "commit-1",
            "--action",
            "writeback",
        ],
    );
    assert_eq!(cleared["data"]["status"], "unchanged");

    let after = fs::read_to_string(store.join("policy-state.json")).expect("policy state");
    assert_eq!(after, original);
}

#[test]
fn policy_explain_ignores_expired_workflow_evidence() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);
    write_policy_config(&store);

    fs::write(
        store.join("policy-state.json"),
        r#"{
  "workflows": {
    "commit-1": {
      "evidence": {
        "actions": ["writeback"],
        "skip_reason": null
      },
      "created_at_unix": 1,
      "updated_at_unix": 1
    }
  }
}"#,
    )
    .expect("policy state should be written");

    let output = run_json_ok(
        &store,
        &[
            "policy",
            "explain",
            "--trigger",
            "on_git_commit",
            "--workflow-key",
            "commit-1",
            "--host",
            "coding-agent",
            "--path",
            "adapters/coding_agent_adapter.py",
        ],
    );

    assert_eq!(output["data"]["decision"], "block");
    assert_eq!(output["data"]["missing_actions"][0], "writeback");
    assert!(
        output["data"]["reasons"][0]
            .as_str()
            .expect("reason should be string")
            .contains("expired")
    );
}

#[test]
fn policy_explain_uses_stored_ttl_instead_of_current_default() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);

    fs::write(
        store.join("policy.toml"),
        r#"
version = 1

[defaults]
scope_strategy = "repo"
evidence_ttl = "task"

[[rules]]
id = "commit-writeback"
trigger = "on_git_commit"
mode = "required"
requires = ["writeback"]
allow_skip = false
on_unsatisfied = "block"

[rules.when]
host = ["coding-agent"]
paths_any = ["adapters/**"]
"#,
    )
    .expect("policy config should be written");

    fs::write(
        store.join("policy-state.json"),
        r#"{
  "workflows": {
    "commit-1": {
      "evidence": {
        "actions": ["writeback"],
        "skip_reason": null
      },
      "evidence_ttl": "manual",
      "created_at_unix": 1,
      "updated_at_unix": 1
    }
  }
}"#,
    )
    .expect("policy state should be written");

    let output = run_json_ok(
        &store,
        &[
            "policy",
            "explain",
            "--trigger",
            "on_git_commit",
            "--workflow-key",
            "commit-1",
            "--host",
            "coding-agent",
            "--path",
            "adapters/coding_agent_adapter.py",
        ],
    );

    assert_eq!(output["data"]["decision"], "allow");
    assert_eq!(output["data"]["matched_rules"][0]["satisfied"], true);
}

#[test]
fn policy_cleanup_filters_by_stored_ttl() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let store = temp_dir.path().join("store");
    let _ = init_store(&store);
    write_policy_config(&store);

    fs::write(
        store.join("policy-state.json"),
        r#"{
  "workflows": {
    "manual-workflow": {
      "evidence": {
        "actions": ["writeback"],
        "skip_reason": null
      },
      "evidence_ttl": "manual",
      "created_at_unix": 1,
      "updated_at_unix": 1
    }
  }
}"#,
    )
    .expect("policy state should be written");

    let cleaned = run_json_ok(
        &store,
        &["policy", "cleanup", "--ttl", "task", "--older-than", "6h"],
    );
    assert_eq!(cleaned["data"]["status"], "cleaned");
    assert!(
        cleaned["data"]["message"]
            .as_str()
            .expect("message")
            .contains("stored `task` entries")
    );

    let explained = run_json_ok(
        &store,
        &[
            "policy",
            "explain",
            "--trigger",
            "on_git_commit",
            "--workflow-key",
            "manual-workflow",
            "--host",
            "coding-agent",
            "--path",
            "adapters/coding_agent_adapter.py",
        ],
    );
    assert_eq!(explained["data"]["decision"], "allow");
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
            "repo:mnemix",
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
