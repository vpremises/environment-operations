# Local OSS inspection

The `oss` subcommands inspect explicitly selected sources without network access,
installation, package scripts, publication or executing inspected repository code.
They run in Linux/WSL, including selected directories mounted from Windows.

```sh
vpremises-security oss repository /sources/library
vpremises-security oss exclusions /sources/library
vpremises-security oss workspace repository /sources/workspace
vpremises-security oss workspace exclusions /sources/workspace
vpremises-security oss package npm /sources/library \
  --archive /private/library.tgz --audit /private/audit.json
vpremises-security oss package cargo /sources/library \
  --archive /private/library-0.1.0.crate --audit /private/audit.json
```

## Repository policy

A version-1 `.github/oss-policy.json` specifies the expected manifest license and
exact reviewed `allowed_vendor_archives` / `allowed_source_files`. Wildcard,
absolute and parent-directory exceptions are rejected. The declared license is
honored; this does not grant rights or verify ownership. Unknown fields fail closed.
Use `--policy /private/policy.json` to explicitly choose a different policy.

Required files are LICENSE, NOTICE, CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md,
`.github/pull_request_template.md` and `.github/ISSUE_TEMPLATE/config.yml`. License
metadata is compared in package.json, Cargo.toml and pyproject.toml. Virtual Cargo
workspaces have no root-package license requirement. Tracked private/generated
paths and unreviewed archives are findings.

Optional `forbidden_extensions` rejects additional explicitly selected suffixes.
Optional `rust_source` rules specify `max_lines`, `english_ascii` and maintained
relative `directories`. Files, including untracked Rust additions, are checked;
symlinks, malformed documents and unreadable sources remain incomplete. No policy
is inferred when repository checks lack a policy file.

## Exclusions and workspace scope

Exclusion checks exercise 30 generated/private path probes and 9 source visibility
probes. They also find visible generated outputs and tracked ignored files. A
scratch Git index disables global ignores, inherited Git overrides and fsmonitor;
the selected repository index is not changed. Missing policies mean no reviewed
source exceptions for this gate. Git command timeouts are ten seconds and output
is bounded to 8 MiB. A subdirectory cannot borrow its parent's Git index.

Workspace commands inspect physical Git checkouts directly under
`organizations/owner/repository`. Empty/inaccessible inventories fail; symlinked
inventory directories are rejected. Old CSV entries are not authoritative. Each
repository retains its result even if another repository cannot be inspected.

## Publication manifests and archives

Manifest checks require reviewed identity, license, GitHub repository metadata,
official registry routes, npm file allowlists, and Cargo numeric toolchains.
Private npm packages, local/path/git/workspace or alternate-registry publication
dependencies, Cargo replacements and private source configuration are findings.
The publication gate is for selected distributable libraries, not internal
workspaces, applications or standalone executables.

Public catalog entries must equal the explicit package policy allowlist. Archive
checks bind package name/version, required legal documents, applicable additional
notices and npm exports/types/bin targets. They reject duplicate names, unsupported
members, links, traversal and private/generated archive paths. Archive manifests
are checked too, so a packed manifest cannot inject a local dependency unnoticed.

Files are never extracted. Limits: 64 MiB compressed, 100 MB decompressed including
TAR headers/padding, 15,000 members, 16 MiB per file, 1 MiB manifest. Gzip EOF/CRC and
trailing data are checked. Input links and changing bytes fail closed.

Without `--archive`, a pass covers manifests only. With an archive, structure alone
remains incomplete. `--audit` requires a pinned local content collector and an
operator-configured root covering the original archive. The tool scans a private
exact-byte TAR copy through Zixcel/Gitleaks, binds hashes before/after, and removes
the content gap only on complete collection. Cargo `.crate` files use the same TAR
adapter. No unrelated neighbor, network or boundary check runs for this gate.
Collector gaps never become a pass. Detection settings remain operator supplied;
exceptions or missing custom words limit detection and must be reviewed separately.

## Receipts and limitations

JSON/JSONL receipts contain stable reason codes, relative locations and evidence
hashes, never matched file contents. Exit codes: 0 passed, 2 incomplete, 3 findings.
Capture stdout on every exit. `external_actions` is always false. A workspace pass
is the aggregate of its selected gate, not security certification.

OSS gates do not prove customer-data absence, copyright ownership, license text
validity, third-party redistribution permission, dependency vulnerability absence,
build behavior, registry retrieval, Git-history safety or CI permissions. Use the
content audit, current dependency advisory tooling, explicit legal review, builds
and publication verification for those separate scopes.

The archive gate requires the shipped [native Gitleaks profile](../examples/oss.gitleaks.toml),
with its digest checked before collection. Copy it to a private configuration
location and set `gitleaks.config_file` in the detector settings to that file.
It extends default Gitleaks rules and preserves the private-key/certificate and
GitHub-credential marker checks. Altered/missing profiles remain incomplete.
Custom word and definition files can supplement that profile through the existing
adapter; secret detection is not reimplemented in Rust policy code.
