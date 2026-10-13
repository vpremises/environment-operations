# OSS source contract: vpremises/environment-operations

Product CLI, credential playground and native helper composition over Crowsi

## Responsibility boundary

Own the named environment configuration/planning or application deployment composition. Generic security and provider mechanisms reuse their managed owners; real profiles/secrets and actual environment activation are separate.

Same-function managed packages take priority when their actual capabilities fit. Do not add an unrelated dependency merely to connect organizations. Preserve upstream LICENSE/NOTICE and immutable release/history evidence; unused compatibility sources belong in Git history, not the supported API.

## Package ownership

| Manifest | Package | Distribution policy |
| --- | --- | --- |
| `apps/environment-plan/Cargo.toml` | `vpremises-environment-plan` | source-only; publish=false |
| `apps/security-inspector/Cargo.toml` | `vpremises-security` | source-only; publish=false |
| `native/windows-custody-helper/Cargo.toml` | `vpremises-windows-custody-helper` | source-only; publish=false |
| `playground/credential-operations/Cargo.toml` | `vpremises-credential-examples` | source-only; publish=false |

Manifest versions and publish flags are declarations, not proof of published availability. Public npm exports and Rust implementation/docs belong to the manifests above; generated dist and sibling checkouts are not source-of-truth implementations.

## Source completion criteria

1. The documented purpose, public exports, validators/schema and actual implementation agree. Remove only confirmed-unused aliases, schemas and compatibility adapters after checking consumers; a shared field name does not imply a shared trust responsibility.
2. Runtime and official dependency resolution work from an independent checkout with committed locks. No local sibling artifact, private registry or ambient owner credential is required for the OSS source path.
3. Supported behavior has reproducible examples/playground and functional tests, including invalid input and relevant failure/cancellation/replay/tenant boundaries. A policy-only CI or planning fixture does not prove external effects.
4. CI runs the declared functional acceptance without disguising skipped/unavailable checks as success. Changed public package bytes receive a new reviewed release; immutable old versions are never overwritten.
5. License/NOTICE, security reporting, usage and the actual supported/unsupported scope stay in the owning repository. The owner operates independently of a central private management checkout.
6. Source acceptance, artifact/registry acceptance and production deployment are separate records. This document defines the target; it does not declare unexecuted tests, unreleased APIs or live-account validation completed.

## Existing verification entrypoints

- [.github/workflows/ci.yml](../.github/workflows/ci.yml)
- [.github/workflows/linux-bundle.yml](../.github/workflows/linux-bundle.yml)
- [.github/workflows/windows-build.yml](../.github/workflows/windows-build.yml)

Use the package scripts/owner CI above and the repository README for exact commands. When schema files exist, their canonical validator and consumer must be checked together; deleting a definition on a text search alone is insufficient.

## Acceptance record

Inventory baseline source: `c5d272c1b3d6155bcf974cf21773a101112e4e0a`. This baseline records observed files and declared interfaces, not a completed audit. Subsequent source changes are accepted at their exact commit with test evidence. Production hosting, account login, secret custody installation and data migration remain outside source-only acceptance.
