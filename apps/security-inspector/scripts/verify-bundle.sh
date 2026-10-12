#!/usr/bin/env bash
# Exercise extracted executables with real collectors and exclusively synthetic fixtures.
set -euo pipefail
umask 077
bundle=$(realpath "$1")
workspace=$(realpath "$2")
root=${3:-"$workspace/audit-root"}
mkdir "$root"
private="$workspace/audit-settings"
"$bundle/vpremises-security" init "$private" "$root" > "$workspace/init.json"
run_audit() {
    local expected=$1 name=$2 code=0
    "$bundle/vpremises-security" audit "$private/audit.json" synthetic-fixture --jsonl > "$workspace/$name.json" || code=$?
    [[ "$code" == "$expected" ]] || { printf 'Unexpected audit exit for %s: %s\n' "$name" "$code" >&2; exit 1; }
    jq -e '.schema == "vpremises-security/report/v2" and .external_actions == false' "$workspace/$name.json" > /dev/null
}
run_audit 2 missing-evidence
"$bundle/tools/crowsi-host-network-sensor" observe | jq \
    '{schema:"crowsi://network/host-network-baseline/v1",baseline_id:"synthetic-run-baseline",
    external_actions:false,signal_trust:"unsigned-local",expected_listeners:.listeners,
    expected_default_routes:.default_routes}' > "$private/baseline.json"
now=$(date -u '+%Y-%m-%dT%H:%M:%S.000Z')
epoch=$(date -u '+%s')
jq -n --arg now "$now" '{schema:"crowsi://network/boundary-input/v1",generated_at:$now,
    external_actions:false,environments:[{id:"synthetic-fixture",label:"Synthetic fixture",provider:"linux",
    source:"synthetic-test-only",status:"observed",isolation_mode:"test-boundary",expected_isolation_mode:"test-boundary",
    project_count:1,network_count:1,instance_count:1,public_ingress:false,management_endpoint_exposed:false}]}' > "$private/boundary.json"
hash=$(sha256sum "$private/boundary.json" | cut -d ' ' -f1)
jq --arg bundle "$bundle" --arg private "$private" --arg hash "$hash" --argjson epoch "$epoch" \
    --slurpfile manifest "$bundle/collectors.json" \
    '.collectors.network.baseline="baseline.json" | .collectors.boundary={
        tool:{executable:($bundle+"/tools/crowsi-boundary-monitor"),sha256:$manifest[0].tools["crowsi-boundary-monitor"].sha256,timeout_seconds:120},
        input:"boundary.json",input_sha256:$hash,observed_at_unix_ms:($epoch*1000),max_age_seconds:300}' \
    "$private/audit.json" > "$private/complete.json"
mv "$private/complete.json" "$private/audit.json"
run_audit 0 clean
# External policy, not a baked-in detector word. The target is harmless synthetic data.
jq -n '[{id:"synthetic-disclosure",category:"word",values:["VPREMISES_TEST_DISCLOSURE"]}]' > "$private/dictionary.json"
printf '%s\n' '.env' > "$root/.gitignore"
printf '%s\n' 'VPREMISES_TEST_DISCLOSURE' > "$root/.env"
run_audit 3 ignored-disclosure
jq -e '.checks[] | select(.id == "content-disclosure") | .finding_count > 0' "$workspace/ignored-disclosure.json" > /dev/null
saved_code=0
bash "$bundle/scripts/run-audit.sh" "$bundle/vpremises-security" "$private/audit.json" synthetic-fixture "$workspace/saved-report.jsonl" || saved_code=$?
[[ "$saved_code" == 3 && "$(stat -c '%a' "$workspace/saved-report.jsonl")" == 600 ]] || exit 1
saved_code=0
bash "$bundle/scripts/run-audit.sh" "$bundle/vpremises-security" "$private/audit.json" synthetic-fixture "$workspace/saved-report.jsonl" || saved_code=$?
[[ "$saved_code" == 1 ]] || exit 1
rm "$root/.env"
cp "$private/audit.json" "$private/good.json"
jq '.collectors.content.tool.sha256=("0"*64)' "$private/good.json" > "$private/audit.json"
run_audit 2 wrong-pin
jq '.collectors.network.baseline="/dev/zero"' "$private/good.json" > "$private/audit.json"
run_audit 2 invalid-baseline
jq '.collectors.boundary.observed_at_unix_ms=0' "$private/good.json" > "$private/audit.json"
run_audit 2 stale-evidence
cp "$private/good.json" "$private/audit.json"
ln -s /etc/passwd "$root/outside-link"
run_audit 2 outside-link
rm "$root/outside-link"
printf '%s\n' 'Bundled collectors: seven audit cases and private-report/no-overwrite cases passed.'
