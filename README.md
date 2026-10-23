# Environment operations

Product composition for environment operations. Reusable policy, custody and observation components belong to Crowsi; this repository owns product CLI configuration, synthetic workflow examples and the Windows helper executable.

| Path | Existing package | Responsibility |
| --- | --- | --- |
| apps/security-inspector | vpremises-security | Product inspection CLI and portable Linux bundle composition |
| apps/environment-plan | vpremises-environment-plan | Operator commands consuming the public environment core and Incus renderer |
| playground/environment-planning | synthetic CLI consumer | Disposable resolve/render/plan/receipt/replay demonstration |
| playground/credential-operations | vpremises-credential-examples | Synthetic credential workflows and fail-closed command boundaries |
| native/windows-custody-helper | vpremises-windows-custody-helper | Standalone Windows executable; independent native signing and acceptance |

```sh
cargo +1.97 test --locked --workspace --all-targets --all-features
cargo +1.97 run --locked -p vpremises-credential-examples -- --help
```

The product uses credential-custody through an exact public Git revision while registry/source publication is reviewed. Security bundle collectors fetch exact security-observations revisions at build time, never at runtime. Product dependencies do not use a private registry or local sibling checkout. Existing package names, wire IDs and publish=false remain unchanged.

CI verifies synthetic inputs, builds Windows helper artifacts and attests main artifacts; the owner promotes reviewed assets to a GitHub Release. It does not enroll real accounts, install services, access customer credentials or enable timers. Windows/WSL identity, native signing and deployment acceptance remain separate owner inputs. Existing app-specific scripts and documentation are preserved under apps/security-inspector. Original repository/revision provenance is retained in source-consolidation.json; current source and license notices remain under their owners.

Environment commands consume the exact public environment-deployment Git revision recorded in Cargo.lock. Run `python3 playground/environment-planning/verify.py` for a disposable end-to-end CLI example. The example records synthetic receipts; an operational receipt needs a separate authenticated execution path.

Environment-plan 0.3.0 also exposes the [finite local maintenance lifecycle](docs/finite-maintenance.md):
plan, explicit terminal consent, one-use reservation, stop/revoke, status and
independent readback. It reuses the environment core ledger and Zixcel bounded
physical intake. It grants no host/provider authority and enables no timer.

## Windows artifact verification

The owner-local windows-build workflow builds the product helper, checks the exact public SDK revision in an ephemeral checkout, tests randomly named synthetic CurrentUser credentials on a fresh Windows runner, verifies source/version/PE/ABI/digests and preserves dependency notices. Main artifacts receive GitHub OIDC build provenance; pull requests do not receive attestation privileges. This does not prove WSL acceptance, customer credential enrollment or Windows Authenticode signing.

[OSS responsibility and source completion](docs/oss-source-contract.md) defines the supported package boundary and required acceptance. It does not declare unexecuted source, distribution or production checks complete.

Dependency updates: [repository policy](docs/dependency-updates.md).
