#!/usr/bin/env bash
# Publish one private local report atomically while preserving the audit exit code.
set -euo pipefail
umask 077
[[ $# == 4 ]] || { printf 'Usage: run-audit.sh EXECUTABLE CONFIG ENVIRONMENT NEW-REPORT\n' >&2; exit 1; }
executable=$1 config=$2 environment=$3 output=$4
directory=$(dirname -- "$output")
[[ ! -L "$directory" && -d "$directory" ]] || exit 1
[[ "$(stat -c '%u' "$directory")" == "$(id -u)" && "$(stat -c '%a' "$directory")" == 700 ]] || exit 1
directory=$(realpath -- "$directory")
name=$(basename -- "$output")
[[ "$name" != '.' && "$name" != '..' && ! -e "$directory/$name" && ! -L "$directory/$name" ]] || exit 1
temporary=$(mktemp "$directory/.vpremises-report.XXXXXX")
trap 'rm -f -- "$temporary"' EXIT
code=0
"$executable" audit "$config" "$environment" --jsonl > "$temporary" || code=$?
# Failed invocations remain failed; never label an invalid document as a report.
[[ "$code" == 0 || "$code" == 2 || "$code" == 3 ]] || exit "$code"
ln -- "$temporary" "$directory/$name"
exit "$code"
