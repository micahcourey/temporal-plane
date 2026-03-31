use std::collections::BTreeMap;

use crate::output::{
    CheckpointResultView, CommandOutput, MemoryDetailView, MemoryListView, MemoryResultView,
    MemorySummaryView, OptimizeResultView, PolicyDecisionView, PolicyRuleEvaluationView,
    RecallEntryView, RecallResultView, RestoreResultView, StatsResultView, StatusView,
    VersionListView,
};

pub(crate) fn render_human(output: &CommandOutput) -> String {
    let mut rendered = String::new();
    match output {
        CommandOutput::Status(view) => render_status(&mut rendered, view),
        CommandOutput::Memory(view) => render_memory(&mut rendered, view),
        CommandOutput::MemoryList(view) => render_memory_list(&mut rendered, view),
        CommandOutput::Recall(view) => render_recall(&mut rendered, view),
        CommandOutput::Policy(view) => render_policy(&mut rendered, view),
        CommandOutput::Checkpoint(view) => render_checkpoint(&mut rendered, view),
        CommandOutput::VersionList(view) => render_version_list(&mut rendered, view),
        CommandOutput::Restore(view) => render_restore(&mut rendered, view),
        CommandOutput::Optimize(view) => render_optimize(&mut rendered, view),
        CommandOutput::Stats(view) => render_stats(&mut rendered, view),
    }
    rendered
}

fn render_policy(buffer: &mut String, view: &PolicyDecisionView) {
    push_line(
        buffer,
        &format!("{} {}: {}", view.command, view.action, view.decision),
    );
    push_line(buffer, &format!("trigger: {}", view.trigger));
    if let Some(workflow_key) = &view.workflow_key {
        push_line(buffer, &format!("workflow_key: {workflow_key}"));
    }
    push_line(buffer, &format!("scope_strategy: {}", view.scope_strategy));
    if !view.required_actions.is_empty() {
        push_line(
            buffer,
            &format!("required_actions: {}", view.required_actions.join(", ")),
        );
    }
    if !view.missing_actions.is_empty() {
        push_line(
            buffer,
            &format!("missing_actions: {}", view.missing_actions.join(", ")),
        );
    }
    for reason in &view.reasons {
        push_line(buffer, &format!("reason: {reason}"));
    }
    if !view.matched_rules.is_empty() {
        push_line(buffer, "matched_rules:");
        for rule in &view.matched_rules {
            render_policy_rule(buffer, rule);
        }
    }
}

fn render_policy_rule(buffer: &mut String, view: &PolicyRuleEvaluationView) {
    push_line(buffer, "---");
    push_line(buffer, &format!("id: {}", view.id));
    push_line(buffer, &format!("mode: {}", view.mode));
    push_line(buffer, &format!("satisfied: {}", view.satisfied));
    push_line(
        buffer,
        &format!("skipped_via_reason: {}", view.skipped_via_reason),
    );
    if !view.required_actions.is_empty() {
        push_line(
            buffer,
            &format!("required_actions: {}", view.required_actions.join(", ")),
        );
    }
    if !view.missing_actions.is_empty() {
        push_line(
            buffer,
            &format!("missing_actions: {}", view.missing_actions.join(", ")),
        );
    }
    for reason in &view.reasons {
        push_line(buffer, &format!("reason: {reason}"));
    }
}

fn render_status(buffer: &mut String, view: &StatusView) {
    push_line(buffer, &format!("{}: {}", view.command, view.status));
    push_line(buffer, &view.message);
    if let Some(path) = &view.path {
        push_line(buffer, &format!("path: {path}"));
    }
    if let Some(schema_version) = view.schema_version {
        push_line(buffer, &format!("schema_version: {schema_version}"));
    }
}

fn render_memory(buffer: &mut String, view: &MemoryResultView) {
    push_line(buffer, &format!("{}: {}", view.command, view.action));
    render_memory_detail(buffer, &view.memory);
}

fn render_memory_list(buffer: &mut String, view: &MemoryListView) {
    push_line(
        buffer,
        &format!("{}: {} result(s)", view.command, view.count),
    );
    if let Some(scope) = &view.scope {
        push_line(buffer, &format!("scope: {scope}"));
    }
    if let Some(query_text) = &view.query_text {
        push_line(buffer, &format!("query: {query_text}"));
    }
    if view.memories.is_empty() {
        push_line(buffer, "(no memories)");
        return;
    }
    for memory in &view.memories {
        push_line(buffer, "---");
        render_memory_summary(buffer, memory);
    }
}

fn render_checkpoint(buffer: &mut String, view: &CheckpointResultView) {
    push_line(buffer, &format!("{}: {}", view.command, view.action));
    push_line(buffer, &format!("name: {}", view.checkpoint.name));
    push_line(buffer, &format!("version: {}", view.checkpoint.version));
    push_line(
        buffer,
        &format!("created_at: {}", view.checkpoint.created_at),
    );
    if let Some(description) = &view.checkpoint.description {
        push_line(buffer, &format!("description: {description}"));
    }
}

fn render_recall(buffer: &mut String, view: &RecallResultView) {
    push_line(
        buffer,
        &format!("{}: {} result(s)", view.command, view.count),
    );
    if let Some(scope) = &view.scope {
        push_line(buffer, &format!("scope: {scope}"));
    }
    if let Some(query_text) = &view.query_text {
        push_line(buffer, &format!("query: {query_text}"));
    }
    push_line(
        buffer,
        &format!("disclosure_depth: {}", view.disclosure_depth),
    );
    render_recall_section(buffer, "pinned_context", &view.pinned_context);
    render_recall_section(buffer, "summaries", &view.summaries);
    render_recall_section(buffer, "archival", &view.archival);
}

fn render_restore(buffer: &mut String, view: &RestoreResultView) {
    push_line(buffer, view.command);
    push_line(buffer, &format!("target_kind: {}", view.target.kind));
    if let Some(name) = &view.target.name {
        push_line(buffer, &format!("target_name: {name}"));
    }
    push_line(
        buffer,
        &format!("restored_version: {}", view.restored_version),
    );
    push_line(
        buffer,
        &format!("previous_version: {}", view.previous_version),
    );
    push_line(
        buffer,
        &format!("current_version: {}", view.current_version),
    );
    if let Some(checkpoint) = &view.pre_restore_checkpoint {
        push_line(buffer, "pre_restore_checkpoint:");
        push_line(buffer, &format!("  name: {}", checkpoint.name));
        push_line(buffer, &format!("  version: {}", checkpoint.version));
    }
}

fn render_version_list(buffer: &mut String, view: &VersionListView) {
    push_line(
        buffer,
        &format!("{}: {} version(s)", view.command, view.count),
    );
    if let Some(scope) = &view.scope {
        push_line(buffer, &format!("scope: {scope}"));
    }
    if view.versions.is_empty() {
        push_line(buffer, "(no versions)");
        return;
    }
    for version in &view.versions {
        push_line(buffer, "---");
        push_line(buffer, &format!("version: {}", version.version));
        push_line(buffer, &format!("recorded_at: {}", version.recorded_at));
        if let Some(checkpoint_name) = &version.checkpoint_name {
            push_line(buffer, &format!("checkpoint: {checkpoint_name}"));
        }
        if let Some(checkpoint_version) = version.checkpoint_version {
            push_line(buffer, &format!("checkpoint_version: {checkpoint_version}"));
        }
        if let Some(summary) = &version.summary {
            push_line(buffer, &format!("summary: {summary}"));
        }
    }
}

fn render_stats(buffer: &mut String, view: &StatsResultView) {
    push_line(buffer, &format!("{}:", view.command));
    if let Some(scope) = &view.stats.scope {
        push_line(buffer, &format!("scope: {scope}"));
    }
    push_line(
        buffer,
        &format!("total_memories: {}", view.stats.total_memories),
    );
    push_line(
        buffer,
        &format!("pinned_memories: {}", view.stats.pinned_memories),
    );
    push_line(
        buffer,
        &format!("version_count: {}", view.stats.version_count),
    );
    if let Some(checkpoint) = &view.stats.latest_checkpoint {
        push_line(buffer, &format!("latest_checkpoint: {checkpoint}"));
    }
}

fn render_optimize(buffer: &mut String, view: &OptimizeResultView) {
    push_line(buffer, view.command);
    push_line(
        buffer,
        &format!("previous_version: {}", view.previous_version),
    );
    push_line(
        buffer,
        &format!("current_version: {}", view.current_version),
    );
    push_line(buffer, &format!("compacted: {}", view.compacted));
    push_line(
        buffer,
        &format!("prune_old_versions: {}", view.prune_old_versions),
    );
    push_line(
        buffer,
        &format!("pruned_versions: {}", view.pruned_versions),
    );
    push_line(buffer, &format!("bytes_removed: {}", view.bytes_removed));
    push_line(
        buffer,
        &format!(
            "retention_minimum_age_days: {}",
            view.retention.minimum_age_days
        ),
    );
    push_line(
        buffer,
        &format!(
            "retention_delete_unverified: {}",
            view.retention.delete_unverified
        ),
    );
    push_line(
        buffer,
        &format!(
            "retention_error_if_tagged_old_versions: {}",
            view.retention.error_if_tagged_old_versions
        ),
    );
    if let Some(checkpoint) = &view.pre_optimize_checkpoint {
        push_line(buffer, "pre_optimize_checkpoint:");
        push_line(buffer, &format!("  name: {}", checkpoint.name));
        push_line(buffer, &format!("  version: {}", checkpoint.version));
    }
}

fn render_memory_summary(buffer: &mut String, memory: &MemorySummaryView) {
    let common = MemoryCommon {
        id: &memory.id,
        scope_id: &memory.scope_id,
        kind: memory.kind,
        title: &memory.title,
        summary: &memory.summary,
        created_at: &memory.created_at,
        updated_at: &memory.updated_at,
        importance: memory.importance,
        confidence: memory.confidence,
        pinned: memory.pinned,
        pin_reason: memory.pin_reason.as_deref(),
    };
    render_memory_common(buffer, &common);
    render_list(buffer, "tags", &memory.tags);
    render_list(buffer, "entities", &memory.entities);
}

fn render_memory_detail(buffer: &mut String, memory: &MemoryDetailView) {
    let common = MemoryCommon {
        id: &memory.id,
        scope_id: &memory.scope_id,
        kind: memory.kind,
        title: &memory.title,
        summary: &memory.summary,
        created_at: &memory.created_at,
        updated_at: &memory.updated_at,
        importance: memory.importance,
        confidence: memory.confidence,
        pinned: memory.pinned,
        pin_reason: memory.pin_reason.as_deref(),
    };
    render_memory_common(buffer, &common);
    push_line(buffer, &format!("detail: {}", memory.detail));
    if let Some(source_session_id) = &memory.source_session_id {
        push_line(buffer, &format!("source_session_id: {source_session_id}"));
    }
    if let Some(source_tool) = &memory.source_tool {
        push_line(buffer, &format!("source_tool: {source_tool}"));
    }
    if let Some(source_ref) = &memory.source_ref {
        push_line(buffer, &format!("source_ref: {source_ref}"));
    }
    render_list(buffer, "tags", &memory.tags);
    render_list(buffer, "entities", &memory.entities);
    render_metadata(buffer, &memory.metadata);
}

fn render_recall_section(buffer: &mut String, label: &str, entries: &[RecallEntryView]) {
    push_line(buffer, &format!("{label}: {}", entries.len()));
    if entries.is_empty() {
        return;
    }
    for entry in entries {
        push_line(buffer, "---");
        push_line(buffer, &format!("layer: {}", entry.layer));
        push_line(buffer, &format!("reasons: {}", entry.reasons.join(", ")));
        render_memory_summary(buffer, &entry.memory);
    }
}

struct MemoryCommon<'a> {
    id: &'a str,
    scope_id: &'a str,
    kind: &'a str,
    title: &'a str,
    summary: &'a str,
    created_at: &'a str,
    updated_at: &'a str,
    importance: u8,
    confidence: u8,
    pinned: bool,
    pin_reason: Option<&'a str>,
}

fn render_memory_common(buffer: &mut String, memory: &MemoryCommon<'_>) {
    push_line(buffer, &format!("id: {}", memory.id));
    push_line(buffer, &format!("scope_id: {}", memory.scope_id));
    push_line(buffer, &format!("kind: {}", memory.kind));
    push_line(buffer, &format!("title: {}", memory.title));
    push_line(buffer, &format!("summary: {}", memory.summary));
    push_line(buffer, &format!("created_at: {}", memory.created_at));
    push_line(buffer, &format!("updated_at: {}", memory.updated_at));
    push_line(buffer, &format!("importance: {}", memory.importance));
    push_line(buffer, &format!("confidence: {}", memory.confidence));
    push_line(buffer, &format!("pinned: {}", memory.pinned));
    if let Some(reason) = memory.pin_reason {
        push_line(buffer, &format!("pin_reason: {reason}"));
    }
}

fn render_list(buffer: &mut String, label: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    push_line(buffer, &format!("{label}: {}", values.join(", ")));
}

fn render_metadata(buffer: &mut String, metadata: &BTreeMap<String, String>) {
    if metadata.is_empty() {
        return;
    }
    push_line(buffer, "metadata:");
    for (key, value) in metadata {
        push_line(buffer, &format!("  {key}={value}"));
    }
}

fn push_line(buffer: &mut String, line: &str) {
    buffer.push_str(line);
    buffer.push('\n');
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::output::{CheckpointView, OptimizeRetentionView, StatsView, VersionView};

    fn demo_memory_detail() -> MemoryDetailView {
        MemoryDetailView {
            id: "memory:1".to_owned(),
            scope_id: "repo:mnemix".to_owned(),
            kind: "decision",
            title: "Freeze the CLI contract".to_owned(),
            summary: "Keep rendering separate from command execution".to_owned(),
            detail: "This memory captures the human-first CLI output boundary.".to_owned(),
            pinned: true,
            pin_reason: Some("Used in every CLI test".to_owned()),
            importance: 90,
            confidence: 95,
            created_at: "1970-01-01T00:16:40Z".to_owned(),
            updated_at: "1970-01-01T00:33:20Z".to_owned(),
            source_session_id: Some("session:cli".to_owned()),
            source_tool: Some("copilot".to_owned()),
            source_ref: Some("docs/mnemix-roadmap.md".to_owned()),
            tags: vec!["milestone-3".to_owned(), "cli".to_owned()],
            entities: vec!["Mnemix".to_owned()],
            metadata: BTreeMap::from([("owner".to_owned(), "cli".to_owned())]),
        }
    }

    fn demo_memory_summary() -> MemorySummaryView {
        MemorySummaryView {
            id: "memory:1".to_owned(),
            scope_id: "repo:mnemix".to_owned(),
            kind: "decision",
            title: "Freeze the CLI contract".to_owned(),
            summary: "Keep rendering separate from command execution".to_owned(),
            pinned: true,
            pin_reason: Some("Used in every CLI test".to_owned()),
            importance: 90,
            confidence: 95,
            created_at: "1970-01-01T00:16:40Z".to_owned(),
            updated_at: "1970-01-01T00:33:20Z".to_owned(),
            tags: vec!["milestone-3".to_owned(), "cli".to_owned()],
            entities: vec!["Mnemix".to_owned()],
        }
    }

    fn demo_recall_entry(layer: &'static str) -> RecallEntryView {
        RecallEntryView {
            layer,
            reasons: vec!["scope_filter", "text_match", "importance_boost"],
            memory: demo_memory_summary(),
        }
    }

    #[test]
    fn status_output_snapshot() {
        let output = CommandOutput::Status(Box::new(StatusView {
            command: "init",
            status: "ok",
            message: "Initialized Mnemix store".to_owned(),
            path: Some("/tmp/plane".to_owned()),
            schema_version: Some(1),
        }));

        assert_snapshot!(render_human(&output), @r"
init: ok
Initialized Mnemix store
path: /tmp/plane
schema_version: 1
");
    }

    #[test]
    fn memory_output_snapshot() {
        let output = CommandOutput::Memory(Box::new(MemoryResultView {
            command: "show",
            action: "displayed memory",
            memory: demo_memory_detail(),
        }));

        assert_snapshot!(render_human(&output), @r"
show: displayed memory
id: memory:1
scope_id: repo:mnemix
kind: decision
title: Freeze the CLI contract
summary: Keep rendering separate from command execution
created_at: 1970-01-01T00:16:40Z
updated_at: 1970-01-01T00:33:20Z
importance: 90
confidence: 95
pinned: true
pin_reason: Used in every CLI test
detail: This memory captures the human-first CLI output boundary.
source_session_id: session:cli
source_tool: copilot
source_ref: docs/mnemix-roadmap.md
tags: milestone-3, cli
entities: Mnemix
metadata:
  owner=cli
");
    }

    #[test]
    fn memory_list_output_snapshot() {
        let output = CommandOutput::MemoryList(Box::new(MemoryListView {
            command: "search",
            scope: Some("repo:mnemix".to_owned()),
            query_text: Some("CLI".to_owned()),
            count: 1,
            memories: vec![demo_memory_summary()],
        }));

        assert_snapshot!(render_human(&output), @r"
search: 1 result(s)
scope: repo:mnemix
query: CLI
---
id: memory:1
scope_id: repo:mnemix
kind: decision
title: Freeze the CLI contract
summary: Keep rendering separate from command execution
created_at: 1970-01-01T00:16:40Z
updated_at: 1970-01-01T00:33:20Z
importance: 90
confidence: 95
pinned: true
pin_reason: Used in every CLI test
tags: milestone-3, cli
entities: Mnemix
");
    }

    #[test]
    fn recall_output_snapshot() {
        let output = CommandOutput::Recall(Box::new(RecallResultView {
            command: "recall",
            scope: Some("repo:mnemix".to_owned()),
            query_text: Some("CLI".to_owned()),
            disclosure_depth: "summary_then_pinned",
            count: 2,
            pinned_context: vec![demo_recall_entry("pinned_context")],
            summaries: vec![demo_recall_entry("summary")],
            archival: Vec::new(),
        }));

        assert_snapshot!(render_human(&output), @r"
recall: 2 result(s)
scope: repo:mnemix
query: CLI
disclosure_depth: summary_then_pinned
pinned_context: 1
---
layer: pinned_context
reasons: scope_filter, text_match, importance_boost
id: memory:1
scope_id: repo:mnemix
kind: decision
title: Freeze the CLI contract
summary: Keep rendering separate from command execution
created_at: 1970-01-01T00:16:40Z
updated_at: 1970-01-01T00:33:20Z
importance: 90
confidence: 95
pinned: true
pin_reason: Used in every CLI test
tags: milestone-3, cli
entities: Mnemix
summaries: 1
---
layer: summary
reasons: scope_filter, text_match, importance_boost
id: memory:1
scope_id: repo:mnemix
kind: decision
title: Freeze the CLI contract
summary: Keep rendering separate from command execution
created_at: 1970-01-01T00:16:40Z
updated_at: 1970-01-01T00:33:20Z
importance: 90
confidence: 95
pinned: true
pin_reason: Used in every CLI test
tags: milestone-3, cli
entities: Mnemix
archival: 0
");
    }

    #[test]
    fn version_and_stats_output_snapshot() {
        let version_output = CommandOutput::VersionList(Box::new(VersionListView {
            command: "history",
            count: 1,
            scope: None,
            versions: vec![VersionView {
                version: 7,
                recorded_at: "1970-01-01T00:50:00Z".to_owned(),
                checkpoint_name: Some("milestone-3".to_owned()),
                checkpoint_version: Some(7),
                summary: Some("Human-first CLI MVP".to_owned()),
            }],
        }));
        let stats_output = CommandOutput::Stats(Box::new(StatsResultView {
            command: "stats",
            stats: StatsView {
                scope: Some("repo:mnemix".to_owned()),
                total_memories: 3,
                pinned_memories: 1,
                version_count: 7,
                latest_checkpoint: Some("milestone-3".to_owned()),
            },
        }));

        assert_snapshot!(render_human(&version_output), @r"
history: 1 version(s)
---
version: 7
recorded_at: 1970-01-01T00:50:00Z
checkpoint: milestone-3
checkpoint_version: 7
summary: Human-first CLI MVP
");
        assert_snapshot!(render_human(&stats_output), @r"
stats:
scope: repo:mnemix
total_memories: 3
pinned_memories: 1
version_count: 7
latest_checkpoint: milestone-3
");
    }

    #[test]
    fn checkpoint_output_snapshot() {
        let output = CommandOutput::Checkpoint(Box::new(CheckpointResultView {
            command: "checkpoint",
            action: "created checkpoint",
            checkpoint: CheckpointView {
                name: "milestone-3".to_owned(),
                version: 7,
                created_at: "1970-01-01T01:06:40Z".to_owned(),
                description: Some("CLI MVP baseline".to_owned()),
            },
        }));

        assert_snapshot!(render_human(&output), @r"
checkpoint: created checkpoint
name: milestone-3
version: 7
created_at: 1970-01-01T01:06:40Z
description: CLI MVP baseline
");
    }

    #[test]
    fn restore_and_optimize_output_snapshots() {
        let restore_output = CommandOutput::Restore(Box::new(RestoreResultView {
            command: "restore",
            target: crate::output::RestoreTargetView {
                kind: "checkpoint",
                name: Some("milestone-5".to_owned()),
                version: 3,
            },
            previous_version: 7,
            restored_version: 3,
            current_version: 8,
            pre_restore_checkpoint: Some(CheckpointView {
                name: "pre-restore-v7".to_owned(),
                version: 7,
                created_at: "1970-01-01T01:16:40Z".to_owned(),
                description: None,
            }),
        }));
        let optimize_output = CommandOutput::Optimize(Box::new(OptimizeResultView {
            command: "optimize",
            previous_version: 8,
            current_version: 9,
            compacted: true,
            prune_old_versions: true,
            pruned_versions: 2,
            bytes_removed: 4096,
            retention: OptimizeRetentionView {
                minimum_age_days: 30,
                delete_unverified: false,
                error_if_tagged_old_versions: true,
            },
            pre_optimize_checkpoint: Some(CheckpointView {
                name: "pre-optimize-v8".to_owned(),
                version: 8,
                created_at: "1970-01-01T01:23:20Z".to_owned(),
                description: None,
            }),
        }));

        assert_snapshot!(render_human(&restore_output), @r"
restore
target_kind: checkpoint
target_name: milestone-5
restored_version: 3
previous_version: 7
current_version: 8
pre_restore_checkpoint:
  name: pre-restore-v7
  version: 7
");
        assert_snapshot!(render_human(&optimize_output), @r"
optimize
previous_version: 8
current_version: 9
compacted: true
prune_old_versions: true
pruned_versions: 2
bytes_removed: 4096
retention_minimum_age_days: 30
retention_delete_unverified: false
retention_error_if_tagged_old_versions: true
pre_optimize_checkpoint:
  name: pre-optimize-v8
  version: 8
");
    }
}
