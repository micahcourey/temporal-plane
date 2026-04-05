# Tasks: Embedding Provider Setup Cli Workflow

## Workstream Goal

Give `mnemix` a first-party CLI workflow for configuring and validating
embedding providers so users can set up either a hosted provider or a local
model and then run real backfills plus semantic retrieval from the terminal.

## Execution Slices

### Planning And Product Boundaries

- [x] Create a repo-native workstream for embedding provider setup
- [ ] Confirm the v1 provider paths to support
- [ ] Confirm where machine-local provider profiles should live
- [ ] Confirm the secret-source policy for environment variables, keychain
  aliases, or other local references
  Current direction: keep store vector metadata inside `.mnemix`, but keep
  provider profiles and secret references outside the store.

### Provider Profiles And Config

- [x] Define the provider profile schema for both cloud and local paths
- [x] Implement profile persistence with explicit non-secret inspection output
- [x] Implement profile listing, showing, updating, and removal
- [x] Ensure secret-bearing fields are redacted from all user-visible output

### Provider Runtime And Validation

- [x] Implement one supported cloud embedding provider
- [x] Implement one supported local model provider
- [x] Implement profile validation and reachability checks
- [x] Verify provider model and dimension compatibility with store settings

### Vector And Retrieval Workflow

- [x] Add a provider-aware CLI setup flow from validation into vector enablement
- [x] Enable `vectors backfill --apply` when a compatible provider is selected
- [x] Add semantic-only and hybrid modes to CLI `search`
- [x] Add semantic-only and hybrid modes to CLI `recall`
- [x] Report clear provenance, fallback, and mismatch states

### Documentation And Verification

- [x] Update the README to replace the current unsupported-provider framing
- [x] Document both the cloud and local setup workflows
- [x] Add targeted tests for profile persistence, redaction, validation, and
  semantic CLI flows
- [x] Run the repo verification gates

## Validation Checklist

- [x] Verify secrets never land in store exports, staged imports, CLI output, or
  test fixtures
- [x] Verify cloud-provider misconfiguration errors remain specific and
  actionable
- [x] Verify local-runtime connection failures remain specific and actionable
- [x] Verify semantic-only commands fail explicitly when no compatible provider
  is available
- [x] Verify hybrid commands explain whether semantic matching actually ran
- [x] Verify `vectors backfill --apply` can run through the CLI with a
  configured provider

## Notes

- This workstream intentionally treats runtime provider setup as machine-local
  operator state, not store data.
- V1 should support one cloud path and one local path well before expanding to
  a broader provider matrix.
- The workflow should reduce the need for custom wrappers while still allowing
  advanced users to bring their own runtime integrations later.
