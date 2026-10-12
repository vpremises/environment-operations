# Mounted-directory audit

Run `vpremises-security audit <audit.json> <opaque-environment-id> [--jsonl]`
in Linux/WSL. The closed configuration uses `vpremises-security/audit/v1`.
Relative paths resolve from its configuration directory; parent traversal and
linked configuration paths are rejected. Use private settings outside source.

## Configuration

After verifying the downloaded ZIP and `SHA256SUMS`, run:

```sh
./vpremises-security init /private/new-settings /mnt/c/Inspection/Selected
./vpremises-security audit /private/new-settings/audit.json workstation --jsonl
```

`init` verifies all bundled helper hashes and creates a new mode-0700 directory
with mode-0600 configuration and an empty private dictionary. Existing directories
are never overwritten. Content inspection is connected immediately; network
baseline and boundary observation remain explicitly unconfigured and incomplete.
Edit `dictionary.json` with external `{id, category, values}` definitions. Optional
Zixcel `words_file`, `definitions_file` and native Gitleaks `config_file` settings
are also supported. No private words are embedded in the executable.

Use [audit.mounted.json](../examples/audit.mounted.json) as the full configuration
reference. Copy the bundled boundary tool path/hash from `collectors.json`, then
supply your reviewed network baseline and genuinely fresh boundary evidence.
Do not turn samples into real evidence by replacing their timestamps.

Each executable must be a physical Linux ELF file. The runner copies verified bytes
to private scratch, executes the held file descriptor, applies a timeout and output
bound, and removes temporary content/receipts. No shell command is supplied by config.

## Local collectors

| Collector | Executable and invocation | Scope |
| --- | --- | --- |
| Content | Zixcel `scan-directory --root ROOT --config SETTINGS --output RECEIPT` | Every selected directory entry; no Git ignore filtering |
| Network | Crowsi host-network sensor `evaluate` with baseline on stdin | Current Linux process network namespace |
| Boundary | Crowsi boundary monitor `evaluate INPUT` | Fresh operator-supplied boundary-input/v1 |

The release bundles the exact tested helper revisions and their notices.
Source builds can still use explicitly configured physical collector paths.
The main repository has no path/git Cargo dependencies and runtime never downloads
or discovers helpers implicitly. Directory inspection does not invoke Git.

Zixcel settings pin Gitleaks and load external common word/RE2 definitions. Keep
private words in those files. Detection stays in Gitleaks; vpremises does not add
its own word matcher, repeat detector rules, or return raw match values.

Directory mode includes `.env`, ignored files, names and supported TAR archives.
Native binaries, unsupported formats, symlinks, unreadable/changed files and Git
administrative entries remain incomplete. Git objects/history are not scanned.
Global observer limits must complete before content is read. Zixcel also enforces
its own file, total-byte, archive-depth and entry bounds. Large roots should be
split explicitly; do not treat an exhausted bound as a clean result.

A network collector without a baseline observes but remains incomplete. Baselines
are unsigned local policy, not authorization to reconfigure networking. A WSL
observation does not prove Windows ingress, firewall or process security.

Boundary input bytes must match `input_sha256`. Its UTC `generated_at` timestamp
must match `observed_at_unix_ms` exactly; accepted time forms are UTC seconds or
milliseconds ending in Z. Future or stale observations are rejected using
`max_age_seconds` (1–86400). No synthesized sample is used as a live observation.
The collector evaluates supplied evidence and does not independently attest it.

## Output and protection

Per-check output contains scope, status, stable reason, finding/gap counts and an
evidence digest. Raw filenames, endpoints, contents and engine stderr stay out of
the aggregate report. `passed` means only all configured scoped checks completed
without findings; missing evidence makes the outcome incomplete. Findings remain
visible as counters even when another check is incomplete.

Inspect a narrow selected mount, not `/` or an entire drive by default. WSL Linux
no-follow semantics and permissions govern mounted access; this tool cannot prove
all Windows filesystem/ACL/reparse-point behavior or offline file availability.
Use least privilege and protect output. The tool is not a sandbox, does not revoke
keys, install hooks, change access rights, scan public websites or remediate files.

## Execution budget and evidence consistency

`collector_budget_seconds` is optional, defaults to 300, and accepts 1-3600 seconds.
It shares a monotonic budget across selected roots and subsequent collectors. An
exhausted budget records incomplete coverage. Per-tool timeouts remain 1-600 seconds
and are shortened to the remaining budget, with one-second granularity. This is
not an OS resource limit or a hard deadline on blocking filesystem I/O or cleanup.

Stdout is bounded to 16 MiB through a pipe and is never staged as a raw file.
Normal exit, timeout and errors reclaim the collector's process group. Trusted
collectors must not daemonize or escape that group; this runner is not a sandbox.
Accepted earlier content findings survive later root failures. Fresh receipt totals
and boundary environment summaries must agree with their detailed evidence.

A baseline must be a bounded physical regular file (at most 1 MiB). Evaluation uses
a private fixed copy and binds its digest into network evidence. Boundary evidence
must remain fresh when evaluation finishes, not only when it starts.

See [security-model.md](security-model.md) for trust assumptions, uncovered areas
and private investigation workflow. `doctor` health describes the executable,
not installed detector readiness or endpoint security.

## Scheduled local reports

Create a private Linux directory for reports; mounted Windows permissions alone
are not sufficient for confidential report storage. Use the provided wrapper:

```sh
mkdir -m 700 /private/reports
bash scripts/run-audit.sh ./vpremises-security /private/settings/audit.json workstation /private/reports/new-report.jsonl
```

The wrapper creates a mode-0600 temporary report, links it atomically under a new
name without overwriting existing reports, and preserves exits 0/2/3. Invalid
invocations do not create a report. Integrate this command with your own cron or
systemd timer and use a unique report filename per invocation. It installs no
service or hook and performs no automatic publication or remediation.
