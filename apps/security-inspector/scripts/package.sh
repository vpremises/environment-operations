#!/usr/bin/env bash
# Build one Linux bundle and verify extraction without installing or publishing.
set -euo pipefail
umask 077
: "${CARGO_TARGET_DIR:?Set an external Cargo target directory}"
: "${RUNNER_TEMP:?Set a private temporary directory}"
target=x86_64-unknown-linux-gnu
version=$(sed -n 's/^version = "\([0-9]*\.[0-9]*\.[0-9]*\)"$/\1/p' Cargo.toml)
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || exit 1
output=$(realpath -m -- "${1:-dist}")
mkdir -p -- "$output"
archive="$output/vpremises-security-$version-$target.zip"
[[ ! -e "$archive" && ! -e "$archive.sha256" ]] || exit 1
flags="--remap-path-prefix=$PWD=/source"$'\x1f'"--remap-path-prefix=$RUNNER_TEMP=/build"$'\x1f'"--remap-path-prefix=$HOME=/toolchain"
export CARGO_ENCODED_RUSTFLAGS="$flags"
cargo build --locked --release --target "$target" --bin vpremises-security
stage=$(mktemp -d "$RUNNER_TEMP/vpremises-package.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT
binary="$CARGO_TARGET_DIR/$target/release/vpremises-security"
install -m 0755 -- "$binary" "$stage/vpremises-security"
cp LICENSE NOTICE README.md SECURITY.md CONTRIBUTING.md THIRD_PARTY_NOTICES.md "$stage/"
cp -r docs schemas licenses "$stage/"
bash scripts/collectors.sh "$stage"
engine_archive="$output/gitleaks_8.30.1_hardened_linux_x64.tar.gz"
[[ ! -e "$engine_archive" && ! -e "$engine_archive.sha256" ]] || exit 1
mv "$stage/engine.tgz" "$engine_archive"
sha256sum "$engine_archive" | cut -d ' ' -f1 > "$engine_archive.sha256"
mkdir "$stage/examples"
cp examples/audit.mounted.json examples/oss.gitleaks.toml "$stage/examples/"
mkdir "$stage/scripts"
cp scripts/run-audit.sh "$stage/scripts/"
jq -n --arg package vpremises-security --arg version "$version" --arg target "$target" \
    --arg revision "${GITHUB_SHA:-local-working-tree}" \
    '{package:$package,version:$version,target:$target,revision:$revision}' > "$stage/manifest.json"
(cd "$stage" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS)
# ZIP preserves the Unix executable bit; validate the actual extracted bundle.
(cd "$stage" && zip -qr "$archive" .)
sha256sum "$archive" | cut -d ' ' -f 1 > "$archive.sha256"
mkdir "$stage/extracted"
unzip -q "$archive" -d "$stage/extracted"
(cd "$stage/extracted" && sha256sum --check --strict SHA256SUMS > /dev/null)
test -x "$stage/extracted/vpremises-security"
"$stage/extracted/vpremises-security" doctor | jq -e '.capabilities.metadata_only == true' > /dev/null
bash scripts/verify-bundle.sh "$stage/extracted" "$stage"
bash scripts/verify-report.sh "$stage/extracted/vpremises-security" "$stage"
printf '%s\n' 'Standalone Linux ZIP, file digests and portable report verified.'
