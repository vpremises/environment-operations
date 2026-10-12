#!/usr/bin/env bash
# An aggregate metadata report must preserve absent collectors as unsupported.
set -euo pipefail
binary=$1
scratch=$2
mkdir "$scratch/observed"
printf '%s\n' 'Synthetic report fixture' > "$scratch/observed/example.txt"
jq -n '{schema_version:"vpremises.observer/v1",roots:[{id:"fixture",path:"observed"}],
    limits:{max_depth:4,max_entries:20,max_total_bytes:10000},
    policy:{metadata_only:true,follow_symlinks:false}}' > "$scratch/observer.json"
code=0
"$binary" report "$scratch/observer.json" synthetic-endpoint --jsonl > "$scratch/report.jsonl" || code=$?
[[ "$code" -eq 2 && $(wc -l < "$scratch/report.jsonl") -eq 1 ]] || exit 1
jq -e '.schema == "vpremises-security/report/v2" and .outcome == "incomplete" and
    .observation.ok == true and .external_actions == false and
    ([.checks[] | select(.status == "unsupported")] | length) == 3' "$scratch/report.jsonl" > /dev/null
! grep -Eq 'example.txt|Synthetic report fixture' "$scratch/report.jsonl"
