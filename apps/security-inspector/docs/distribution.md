# Standalone Linux/WSL executable

Download the Linux x64 ZIP and adjacent SHA-256 file from
[GitHub Releases](https://github.com/vpremises/environment-operations/releases).
No native Windows build, package registry, container runtime or Rust installation
is required to run the main executable. Ubuntu 24.04-compatible glibc is required.
The ZIP includes the exact reviewed Zixcel/Crowsi ELF collectors and SHA-pinned
Gitleaks engine. No compiler, Git checkout or runtime download is needed for
directory auditing. JSON setup scripts use the standard Ubuntu `jq` utility.

```sh
version=0.1.4
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

Merge reviewed source through protected main-branch PR checks. The `linux-bundle.yml` workflow builds and verifies the executable bundle; main artifacts receive GitHub OIDC build provenance. Pull requests do not receive attestation permissions. An owner must independently verify the exact repo/workflow/source/ref and artifact hashes before promotion.

Use the component tag `security-inspector-v<version>` on that exact main revision. The owner promotes reviewed artifacts to a GitHub Release; this workflow does not publish automatically. Existing releases or a tag on another source are rejected by the promotion helper. Component releases use explicit tags rather than a shared monorepo `latest` download URL.

CI uses real collectors with harmless synthetic disclosure and boundary fixtures.
This verifies tool operation, not the runner's overall security or real host
isolation. Review the downloaded release's checksum and `SHA256SUMS`, then run
`doctor`, `init` and an explicitly configured audit on the deployment machine.
Reports and private settings must stay outside the extracted bundle.

The release-request parser requires an authorized owner and `/release security-inspector-v<version> <exact-main-sha>`. It validates a request; no automatic comment-triggered publisher is installed in this repository. Actual promotion uses the reviewed artifact and explicit component tag.
