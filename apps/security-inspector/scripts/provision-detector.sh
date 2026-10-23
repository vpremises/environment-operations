#!/usr/bin/env bash
# Product composition only: owner fetch, audit and bounded provision stay in Zixcel.
set -euo pipefail
checkout=$(realpath "$1")
stage=$(realpath "$2")
sources=$(realpath scripts/collectors.json)
: "${RUNNER_TEMP:?}" "${CARGO_TARGET_DIR:?}"
# Both the caller pin and the immutable owner source must identify the same bytes.
if ! jq -e --slurpfile owner "$checkout/tests/gitleaks-pin.json" '
    .gitleaks.url == $owner[0].url and
    .gitleaks.archive_sha256 == $owner[0].archive_sha256 and
    .gitleaks.binary_sha256 == $owner[0].executable_sha256 and
    .gitleaks.source == $owner[0].source and
    .gitleaks.build == $owner[0].build' "$sources" >/dev/null; then
    echo '::error::Product and detector-owner pins disagree; refusing provisioning.' >&2
    exit 1
fi
evidence="$RUNNER_TEMP/inspector-detector-evidence"
[[ ! -e "$evidence" && ! -L "$evidence" ]]
mkdir -m 700 "$evidence"
(
    cd "$checkout"
    bash scripts/fetch-published-detector.sh "$stage/published-engine"
    cp "$stage/published-engine/provenance.json" "$evidence/provenance.json"
    if ! bash scripts/audit-published-detector.sh "$evidence/audit"; then
        echo '::error::Published detector audit failed; inventory and findings retained.' >&2
        if [[ -f "$evidence/audit/vulnerabilities.jsonl" ]]; then
            jq -c 'select(.finding) | .finding | {osv,trace}' "$evidence/audit/vulnerabilities.jsonl" >&2
        fi
        exit 1
    fi
    "$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/zixcel-repository-security" provision-gitleaks         --archive "$stage/published-engine/engine.tar.gz" --output-directory "$stage/engine"
)
install -m 755 "$stage/engine/gitleaks" "$stage/tools/gitleaks"
notice="$stage/licenses/collectors/gitleaks"
mkdir "$notice"
cp "$stage/engine/LICENSE" "$stage/engine/Go-runtime-LICENSE" "$notice/"
cp -r "$stage/engine/licenses" "$notice/dependencies"
cp "$evidence/provenance.json" "$notice/BUILD-PROVENANCE.json"
jq -s '{scanner:.[0].config.scanner_version,database:.[0].config.db,
    database_last_modified:.[0].config.db_last_modified,
    affected_compiled_packages:([.[]|select(.finding.trace[0].package)]|length),
    module_notices:([.[]|select(.finding and (.finding.trace[0].package==null))|.finding.osv]|unique)}'     "$evidence/audit/vulnerabilities.jsonl" > "$notice/ENGINE-SECURITY.json"
mv "$stage/published-engine/engine.tar.gz" "$stage/engine.tgz"
rm -rf -- "$stage/engine" "$stage/published-engine"
printf '%s\n' 'Published detector: caller/owner pins, provenance, fresh binary audit and bounded provisioning verified.'
