# Standalone Linux/WSL executable

Download the Linux x64 ZIP and adjacent SHA-256 file from
[GitHub Releases](https://github.com/vpremises/environment-operations/releases).
No native Windows build, package registry, container runtime or Rust installation
is required to run the main executable. Ubuntu 24.04-compatible glibc is required.
The ZIP includes the exact reviewed Zixcel/Crowsi ELF collectors and SHA-pinned
Gitleaks engine. No compiler, Git checkout or runtime download is needed for
directory auditing. JSON setup scripts use the standard Ubuntu `jq` utility.

```sh
version=0.1.5
archive=vpremises-security-${version}-x86_64-unknown-linux-gnu.zip
printf '%s  %s\n' "$(tr -d '\r\n' < "$archive.sha256")" "$archive" | sha256sum --check --strict
unzip "$archive" -d vpremises-security
cd vpremises-security
sha256sum --check SHA256SUMS
./vpremises-security doctor
./vpremises-security init /private/new-settings /mnt/c/Inspection/Selected
./vpremises-security audit /private/new-settings/audit.json workstation --jsonl
```

Keep configuration and reports outside the bundle. Report outcome determines exit
code: 0 passed, 2 incomplete, 3 findings; invalid input returns 1. Preserve stdout.
Keep license notices with redistributed executables. Checksums identify bytes;
they do not replace trusted source review or signatures.

## CI and release

Linux-only CI formats, lints, tests, enforces Rust file limits, builds the main
executable and fixed public helper revisions, and exercises the extracted ZIP.
The bundle contains executable tools, source/version manifests, checksums,
third-party notices, schemas and usage documentation. All source revisions and
engine archive digests are fixed in `scripts/collectors.json`.

Merge reviewed source through protected main-branch PR checks. The `linux-bundle.yml` workflow builds and verifies the executable bundle; main artifacts receive GitHub OIDC build provenance. The main job also verifies archive attestations against the exact repository, workflow, main ref and source digest and retains the verification output. Pull requests do not receive attestation permissions. An owner must independently verify the exact repo/workflow/source/ref and artifact hashes before promotion.

Use the component tag `security-inspector-v<version>` on that exact main revision. The owner promotes reviewed artifacts to a GitHub Release; this workflow does not publish automatically. Existing releases or a tag on another source are rejected by the promotion helper. Component releases use explicit tags rather than a shared monorepo `latest` download URL.

CI uses real collectors with harmless synthetic disclosure and boundary fixtures.
This verifies tool operation, not the runner's overall security or real host
isolation. Review the downloaded release's checksum and `SHA256SUMS`, then run
`doctor`, `init` and an explicitly configured audit on the deployment machine.
Reports and private settings must stay outside the extracted bundle.

The release-request parser requires an authorized owner and `/release security-inspector-v<version> <exact-main-sha>`. It validates a request; no automatic comment-triggered publisher is installed in this repository. Actual promotion uses the reviewed artifact and explicit component tag.

## Detector ownership

The product consumes Zixcel’s immutable published detector, rather than rebuilding
its historical Go recipe. The exact owner source, archive/executable hashes and
Go runtime build are pinned in `scripts/collectors.json`; the owner pin must agree
before any detector is accepted. The owner fetch helper verifies GitHub build
provenance, and its audit helper rechecks the actual published binary against the
current vulnerability database. Product packaging reuses the owner’s bounded
Rust provisioner to preserve licenses and reject malformed archives.

A fresh binary audit is required for every bundle build. Failure retains the
structured finding/inventory evidence and stops packaging; no advisory allowlist
or module-only fallback is added. Go is used only by the owner’s CI audit tool,
not to rebuild the detector or as an installed product dependency.
