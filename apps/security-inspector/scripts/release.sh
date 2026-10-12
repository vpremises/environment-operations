#!/usr/bin/env bash
# Publish verified executable bundles for an existing operator-approved version tag.
set -euo pipefail
umask 077
: "${RUNNER_TEMP:?Missing ephemeral runner directory}"
: "${GH_TOKEN:?Missing workflow token}"
: "${GITHUB_REPOSITORY:?Missing repository identity}"
: "${RELEASE_TAG:=${GITHUB_REF_NAME:?Missing release tag}}"
: "${GITHUB_SHA:?Missing source revision}"
[[ "$RELEASE_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || exit 1
version=$(sed -n 's/^version = "\([0-9]*\.[0-9]*\.[0-9]*\)"$/\1/p' Cargo.toml)
[[ "$RELEASE_TAG" == "v$version" ]] || exit 1
[[ "$(git rev-parse HEAD)" == "$GITHUB_SHA" ]] || exit 1
[[ "$(git rev-parse "$RELEASE_TAG^{commit}")" == "$GITHUB_SHA" ]] || exit 1
git merge-base --is-ancestor "$GITHUB_SHA" origin/main
assets=()
for target in x86_64-unknown-linux-gnu; do
    archive="dist/vpremises-security-$version-$target.zip"
    [[ -f "$archive" && ! -L "$archive" && -f "$archive.sha256" && ! -L "$archive.sha256" ]] || exit 1
    expected=$(tr -d '\r\n' < "$archive.sha256")
    [[ "$expected" =~ ^[a-f0-9]{64}$ ]] || exit 1
    printf '%s  %s\n' "$expected" "$archive" | sha256sum --check --strict
    unzip -p "$archive" manifest.json | jq -e --arg version "$version" --arg revision "$GITHUB_SHA" --arg target "$target" \
        '.package == "vpremises-security" and .version == $version and .revision == $revision and .target == $target' > /dev/null
    assets+=("$archive" "$archive.sha256")
done
engine="dist/gitleaks_8.30.1_hardened_linux_x64.tar.gz"
[[ -f "$engine" && ! -L "$engine" && -f "$engine.sha256" && ! -L "$engine.sha256" ]] || exit 1
expected=$(jq -r '.gitleaks.archive_sha256' scripts/collectors.json)
[[ "$(tr -d '\r\n' < "$engine.sha256")" == "$expected" ]] || exit 1
printf '%s  %s\n' "$expected" "$engine" | sha256sum --check --strict
assets+=("$engine" "$engine.sha256")
# Refuse to modify an existing release, including a partially uploaded draft.
error_file=$(mktemp "$RUNNER_TEMP/vpremises-release.XXXXXX")
trap 'unlink -- "$error_file"' EXIT
if gh api "repos/$GITHUB_REPOSITORY/releases/tags/$RELEASE_TAG" > /dev/null 2> "$error_file"; then
    printf '%s\n' 'Release already exists; review it manually' >&2
    exit 1
fi
grep -Fq 'HTTP 404' "$error_file" || exit 1
notes="Standalone Linux/WSL executable. Download the matching ZIP and SHA-256 file, verify the digest, and extract the bundle. No package registry, container runtime, or Rust installation is required. Source revision: $GITHUB_SHA."
gh release create "$RELEASE_TAG" "${assets[@]}" \
    --repo "$GITHUB_REPOSITORY" --verify-tag \
    --title "vpremises-security $version" --notes "$notes"
