#!/usr/bin/env bash
# Build reviewed public helper revisions; runtime never fetches source or engines.
set -euo pipefail
stage=$1
: "${RUNNER_TEMP:?}" "${CARGO_TARGET_DIR:?}"
sources=$(realpath scripts/collectors.json)
mkdir -p "$stage/tools" "$stage/licenses/collectors"
manifest="$stage/collectors.json"
jq -n '{schema:"vpremises-security/collectors/v1",tools:{}}' > "$manifest"
while IFS=$'\t' read -r name repository revision; do
    [[ "$name" =~ ^[a-z0-9-]+$ && "$repository" =~ ^[a-z0-9-]+/[a-z0-9-]+$ && "$revision" =~ ^[a-f0-9]{40}$ ]] || exit 1
    checkout="$stage/source-$name"
    git init -q "$checkout"
    git -C "$checkout" fetch -q --depth 1 "https://github.com/$repository.git" "$revision"
    git -C "$checkout" checkout -q --detach FETCH_HEAD
    [[ "$(git -C "$checkout" rev-parse HEAD)" == "$revision" ]] || exit 1
    if [[ -n "${CARGO_AUDIT_BIN:-}" ]]; then
        # Populate every pinned registry entry before offline yank checks.
        cargo fetch --locked --manifest-path "$checkout/Cargo.toml"
        "$CARGO_AUDIT_BIN" audit --no-fetch --deny warnings --file "$checkout/Cargo.lock"
    fi
    # Use each helper's lockfile and reviewed toolchain; no shared workspace paths.
    (cd "$checkout" && cargo build --locked --release --target x86_64-unknown-linux-gnu --bin "$name")
    install -m 0755 "$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/$name" "$stage/tools/$name"
    if [[ "$name" == zixcel-repository-security ]]; then
        bash scripts/provision-detector.sh "$checkout" "$stage"
    fi
    notice="$stage/licenses/collectors/$name"
    mkdir "$notice"
    for file in LICENSE LICENSE-MIT NOTICE THIRD_PARTY_NOTICES.md; do
        [[ ! -f "$checkout/$file" ]] || cp "$checkout/$file" "$notice/"
    done
    # Include exact registry dependency notices even when the helper has no notice index.
    (cd "$checkout" && cargo metadata --locked --format-version 1) | \
        jq -r '.packages[] | select(.source != null) | .manifest_path' | sort -u | \
        while IFS= read -r dependency; do
            directory=$(dirname "$dependency")
            package=$(basename "$directory")
            mkdir -p "$notice/dependencies/$package"
            (cd "$directory" && find . -maxdepth 2 -type f \( -iname 'license*' -o -iname 'copying*' -o -iname 'notice*' \) \
                -exec cp --parents -t "$notice/dependencies/$package" {} +)
        done
    hash=$(sha256sum "$stage/tools/$name" | cut -d ' ' -f1)
    jq --arg name "$name" --arg repository "$repository" --arg revision "$revision" --arg sha "$hash" \
        '.tools[$name]={repository:$repository,revision:$revision,sha256:$sha}' "$manifest" > "$manifest.next"
    mv "$manifest.next" "$manifest"
    rm -rf -- "$checkout"
done < <(jq -r '.sources[]|[.name,.repository,.revision]|@tsv' "$sources")
hash=$(sha256sum "$stage/tools/gitleaks" | cut -d ' ' -f1)
[[ "$hash" == "$(jq -r '.gitleaks.binary_sha256' "$sources")" ]] || exit 1
printf '%s  %s\n' "$(jq -r '.gitleaks.archive_sha256' "$sources")" "$stage/engine.tgz" | sha256sum -c --strict
jq --arg sha "$hash" --slurpfile sources "$sources" \
    '.tools.gitleaks={repository:"gitleaks/gitleaks",revision:$sources[0].gitleaks.revision,version:$sources[0].gitleaks.version,sha256:$sha}' \
    "$manifest" > "$manifest.next"
mv "$manifest.next" "$manifest"
