# Security model and coverage

This tool is a read-only, scoped Linux/WSL audit coordinator. It is not endpoint
protection, a sandbox, an exposure inventory service or a signed attestation.
A `passed` result applies only to the selected roots and accepted local evidence
at the time of inspection. It does not establish that data has never leaked.

## Trust boundaries

Trust the Linux kernel, standard system programs, the invoking account, reviewed
SHA-256-pinned collector binaries and approved external detector definitions.
Use an unprivileged account and a private configuration/output directory. Local
programs with the same identity can change inputs, inspect memory or interfere
with subprocesses; hashes do not authorize arbitrary binaries or constrain them.
Do not load an executable merely because someone supplied its matching hash.

Source files, directory entries and collector documents are potentially malformed.
Configuration and baseline reads use descriptor-anchored no-follow access,
regular-file checks, byte limits and in-read change detection. Baselines are copied
before evaluation and their digest contributes to the evidence digest. This is an
identity check, not baseline approval or proof of its correctness.

## Implemented controls

| Boundary | Control | Residual limit |
| --- | --- | --- |
| Content | Local Gitleaks via pinned Zixcel; hidden/ignored entries included | Detector definitions determine detection coverage |
| Evidence | Contract, exit status, finding totals and environment totals checked | Trusted helpers can still produce incorrect but consistent evidence |
| Multiple roots | Completed findings survive later failures | No atomic snapshot across changing mounted files |
| Execution | Private pinned ELF copy, cleared environment, process group and deadline | No sandbox, cgroup, seccomp or protection from escaped process groups |
| Stdout | Bounded pipe read; no raw stdout file | Collector-owned files and total memory/CPU are not OS-resource-limited |
| Runtime | Shared monotonic collector budget, default 300 seconds | Seconds granularity; filesystem I/O and cleanup can exceed the budget |
| Boundary | Exact input/time pin, freshness before and after evaluation | Operator-supplied evidence is not independently attested |
| Reports | Counts, stable reasons and digests; no raw matched values or paths | Operator-selected IDs and hashes should still be treated as private |

Content receipts are validated as fresh scans: reviewed findings cannot silently
turn detection into success. Store only per-root receipt hashes while aggregating,
so up to 256 large private receipts are not retained in memory simultaneously.
Malformed, missing, unknown or inconsistent evidence remains incomplete. Known
findings remain counted even when a later root is incomplete.

`doctor` verifies executable capabilities only. `init` checks the bundled collector
digests and creates private external configuration; neither command validates the
endpoint. `doctor` does not install collectors,
validate their settings or declare that an endpoint is secure.

## Coverage requiring separate evidence or tools

| Area | Current coverage | Required approach |
| --- | --- | --- |
| Windows host ACLs, reparse points, processes and firewall | Not established by WSL-mounted files | Native host evidence and separately validated adapters |
| Published repositories, registries and websites | Not inspected by directory audit | Explicit remote inventory and approved read-only inspection |
| Git history and deleted files | Not inspected in directory mode | Separate history scan, including intended publication refs |
| Vulnerable dependencies and semantic code flaws | Not detected by secret scanning | Lockfile-aware advisory checks, SAST and human review |
| Unsupported binaries, archives, links and inaccessible files | Recorded as gaps | Explicit supported decoder/evidence; never waive silently |
| Prevention, key revocation and network enforcement | Not implemented | Separate approved remediation/enforcement tools |
| Continuous monitoring, hooks and scheduling | One-shot CLI only | Operator-controlled scheduler; validate each invocation and exit status |
| Report integrity, origin and retention | Local unsigned JSON/JSONL | Protected local storage; signatures/retention only under explicit policy |

The selected root must exclude private scanner configuration and output. Define
narrow roots; avoid scanning an entire drive. Configuring three collectors does
not extend their scope to all endpoint assets.

## Operation and investigation

Use `scripts/run-audit.sh` for atomic mode-0600 report creation without overwriting
existing files, or protect stdout destination before starting a scheduled run. Use a private directory,
restrictive umask and a fresh output file. Do not ignore exit 2 (incomplete) or
exit 3 (findings), and do not publish the report automatically. The tool installs
no scheduler, hooks or background service implicitly.

Aggregate output deliberately omits detailed findings. For local investigation,
run Zixcel `scan-directory` on the same selected root and external settings with a
fresh private `--output` receipt outside the source. Inspect that receipt locally;
never paste credentials, private words or matched values into public issues.
If inputs have changed, this second invocation is new evidence, not reproduction
of the original snapshot. Remediation and allowlisting require their own review.

## Design references

Rust's [Child lifecycle contract](https://doc.rust-lang.org/std/process/struct.Child.html)
requires explicit process cleanup; dropping a plain child handle is insufficient.
The [OWASP logging guidance](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
explains the handling of sensitive values and access-controlled security logs.
These references inform the boundaries above; they do not certify this implementation.

Directory inventories hold no-follow directory descriptors rather than reopening
mutable pathnames for enumeration. At most 256 pending directory descriptors are
retained per walk; wider pending trees become incomplete and should be split into
narrower explicit roots. This prevents descriptor exhaustion, but does not create
an atomic filesystem snapshot or detect every change between separate invocations.

## Build-time dependency inspection

CI checks all Rust lockfiles against current RustSec advisories with warnings
rejected. The bundled Gitleaks engine retains reviewed upstream detector source
with patched Go/module versions. Its upstream tests and binary package/symbol
inspection run before packaging. Linker symbols remain available so the scanner
cannot silently fall back to approximate whole-module results on stripped binaries.
The module-only OpenPGP notice is retained: those deprecated packages are absent
from the engine's imports and linked symbols. No advisory ID is ignored.
These build checks apply to distributed tools, not every application under a
selected audit root. Runtime directory inspection remains disclosure inspection.
