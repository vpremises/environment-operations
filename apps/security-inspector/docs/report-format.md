# Scoped local audit report

Contract: `vpremises-security/report/v2`.
Schema: [security-report.v2.schema.json](../schemas/security-report.v2.schema.json).
JSONL contains the same UTF-8 JSON object on one line, followed by a newline.

| Field | Meaning |
| --- | --- |
| `schema`, `tool_version` | Report and executable identity |
| `environment_id` | Operator-selected opaque identifier, never a hostname |
| `operating_system` | Linux process environment |
| `started_at_unix_ms`, `finished_at_unix_ms` | UTC invocation timestamps |
| `configuration_sha256` | Effective resolved configuration digest |
| `outcome` | passed, findings, or incomplete within declared scopes |
| `external_actions` | Always false; no remote upload or host enforcement |
| `checks` | Explicit status, reason, scope, finding/gap counts and evidence digest |
| `observation` | Aggregate metadata with no inspected names or file bodies |

A check is completed only when its required evidence is available and accepted.
Missing collectors, stale/pin-mismatched input, unsafe files, timeouts, malformed
output and unavailable baselines remain incomplete. Unsupported formats are gaps.
`report` without collectors records unsupported checks rather than inventing results.

Any incomplete check makes the overall outcome incomplete; finding counters remain
visible. Otherwise any finding yields findings, and all clean completed checks yield
passed. Network scope is the current Linux namespace; isolation scope is supplied
evidence. No outcome certifies an entire Windows host or public exposure globally.

The aggregate report omits inspected paths, names, contents, endpoints and raw engine
errors. Opaque environment/root IDs must not themselves contain personal names.
Evidence digests identify private collector results, not signed attestations.
Definitions, input source identity and scoped coverage remain important when comparing
reports. Do not turn a missing check or unknown contract version into success.

## OSS receipts

The `oss` subcommands use [the OSS gate schema](../schemas/oss-report.schema.json),
with relative filenames and manifest-field locations. They do not share the
path-free endpoint report contract above. Invalid inputs use
`vpremises-security/oss-error/v1` with a stable code and incomplete status.
See [OSS inspection](oss.md) for scope and exit semantics.
