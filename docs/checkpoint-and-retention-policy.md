# Mnemix Checkpoint and Retention Policy

**Status:** draft implementation guidance

## Purpose

This document records the default naming and safety policy for checkpoints, restore, and optimize flows in Mnemix v1.

## Checkpoint naming policy

### User-created checkpoints

User-created checkpoints should be short, human-meaningful names such as:

- `milestone-5`
- `before-import`
- `release-baseline`
- `session-end-2026-03-08`

### Automatic checkpoints

Automatic safety checkpoints use operation-specific prefixes and the current version number:

- `pre-restore-v<version>`
- `pre-optimize-v<version>`
- `pre-import-v<version>`
- `pre-cleanup-v<version>`

If the current version already has a checkpoint, Mnemix reuses the existing checkpoint instead of creating a duplicate version tag.

## Conservative retention defaults

The default retention policy is intentionally conservative:

- keep recent history generously
- require explicit prune intent
- protect all checkpointed versions
- fail pruning when tagged old versions would be removed
- do not delete unverified files by default
- create automatic safety checkpoints before restore and optimize

## Optimize behavior

`optimize` is not treated as a silent background detail.

Current behavior:

- compaction runs explicitly
- index optimization is attempted explicitly
- pruning old versions is disabled unless requested
- prune requests inherit the configured retention threshold
- tagged old versions are protected by default

## Import behavior

Import remains a staged feature, but the retention contract already reserves a pre-import checkpoint policy so future import flows preserve recoverability.

## Product rule

Cleanup is a recoverability decision, not only a storage decision.

Users should be able to inspect history before destructive operations and rely on checkpoints to preserve meaningful milestones.
