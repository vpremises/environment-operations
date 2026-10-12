# Standalone Linux/WSL executable

Download the Linux x64 ZIP and adjacent SHA-256 file from
[GitHub Releases](https://github.com/vpremises/vpremises-security/releases).
No native Windows build, package registry, container runtime or Rust installation
is required to run the main executable. Ubuntu 24.04-compatible glibc is required.
The ZIP includes the exact reviewed Zixcel/Crowsi ELF collectors and SHA-pinned
Gitleaks engine. No compiler, Git checkout or runtime download is needed for
directory auditing. JSON setup scripts use the standard Ubuntu `jq` utility.

```sh
version=0.1.3
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

Merge reviewed source through protected main-branch PR checks. Push an approved
`v<version>` tag on that main revision to run `Standalone Release`. A manual
workflow invocation on the same tag is also supported. The workflow builds and
verifies the bundle before publishing a GitHub Release, verifies its manifest
revision/version/target and refuses to change an existing release. It never
creates or moves a tag. Only the publish job has `contents: write`, using the
short-lived `GITHUB_TOKEN`. No GHCR or registry credentials are used.

CI uses real collectors with harmless synthetic disclosure and boundary fixtures.
This verifies tool operation, not the runner's overall security or real host
isolation. Review the downloaded release's checksum and `SHA256SUMS`, then run
`doctor`, `init` and an explicitly configured audit on the deployment machine.
Reports and private settings must stay outside the extracted bundle.

An authorized `coela-oss` operator may also request publication on a repository PR with `/release v<version> <exact-main-sha>`. The request must select the current main source and its declared package version. The promotion workflow reruns the complete product/bundle CI, creates an immutable tag on that exact source using the short-lived workflow token, and invokes the same verified release script. Existing tags on another source and existing releases are rejected. Local registry or GitHub credentials are not needed.
