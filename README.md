# Environment operations

Product composition for environment operations. Reusable policy, custody and observation components belong to Crowsi; this repository owns product CLI configuration, synthetic workflow examples and the Windows helper executable.

| Path | Existing package | Responsibility |
| --- | --- | --- |
| apps/security-inspector | vpremises-security | Product inspection CLI and portable Linux bundle composition |
| playground/credential-operations | vpremises-credential-examples | Synthetic credential workflows and fail-closed command boundaries |
| native/windows-custody-helper | vpremises-windows-custody-helper | Standalone Windows executable; independent native signing and acceptance |

```sh
cargo +1.97 test --locked --workspace --all-targets --all-features
cargo +1.97 run --locked -p vpremises-credential-examples -- --help
```

The product uses credential-custody through an exact public Git revision while registry/source publication is reviewed. Security bundle collectors fetch exact security-observations revisions at build time, never at runtime. Product dependencies do not use a private registry or local sibling checkout. Existing package names, wire IDs and publish=false remain unchanged.

CI verifies synthetic inputs only. It does not enroll accounts, install services, access real credentials, enable timers or publish native releases. Windows/WSL identity, native signing and deployment acceptance remain separate owner inputs. Existing app-specific scripts and documentation are preserved under apps/security-inspector; native release workflows are not automatically enabled. Original public source history is preserved under docs/source-history.

## Windows artifact verification

The owner-local windows-build workflow builds the product helper, checks the exact public SDK revision in an ephemeral checkout, tests randomly named synthetic CurrentUser credentials on a fresh Windows runner, verifies source/version/PE/ABI/digests and preserves dependency notices. Main artifacts receive GitHub OIDC build provenance; pull requests do not receive attestation privileges. This does not prove WSL acceptance, customer credential enrollment or Windows Authenticode signing.
