# Finite local maintenance consent and readback

`vpremises-environment-plan` 0.3.0 consumes the exact accepted public environment
core revision in Cargo.lock. Core consent/state transitions remain in
`vpremises/environment-deployment`; this product owns the operator CLI and
synthetic composition. Bounded physical file intake reuses the existing
`zixcel-bounded-io` implementation instead of a second file-opening library.
Private real profiles stay in `environment-configurations`. Crowsi remains the
owner of credential custody and real execution authorization.

These commands record local consent/reservations and observe supplied local
state files. They do **not** run a shell command, update a host, call a provider,
approve npm stages/deployments or enable a timer. Every result explicitly keeps
`execution_authorized=false`. A reservation is not a credential or Crowsi grant.
An approved real host adapter must independently enforce privileges, target CAS,
bounded execution, cancellation and authenticated readback before using this
lifecycle for effects. Existing custody/host acceptance holds are not resumed.

## Prepare a finite candidate

Use the normal `resolve` command to produce reviewed `RESOLVED.json`. Prepare
the exact reviewed source, release artifact and inventory files. A small draft
contains only `id`, `valid_seconds`, `max_runtime_seconds` and `steps`. Each step
has an ID, product operation name and distinct before/after state SHA256 values.
There is no executable path or arbitrary command field.

```json
{
  "id": "reviewed-maintenance-001",
  "valid_seconds": 300,
  "max_runtime_seconds": 60,
  "steps": [
    {
      "id": "reviewed-step",
      "operation": "apply-reviewed-release",
      "before_sha256": "<64 lowercase hex characters for the reviewed before-state>",
      "after_sha256": "<64 lowercase hex characters for the reviewed desired state>"
    }
  ]
}
```

Replace both illustrative digest placeholders with actual reviewed state hashes;
the CLI rejects the example as written. From this repository:

```sh
cargo +1.97 run --locked -p vpremises-environment-plan -- maintenance-plan DRAFT.json RESOLVED.json SOURCE ARTIFACT INVENTORY > CANDIDATE.json
cargo +1.97 run --locked -p vpremises-environment-plan -- maintenance-approve /ABSOLUTE/PRIVATE/ledger.sqlite3 CANDIDATE.json RESOLVED.json SOURCE ARTIFACT INVENTORY
```

The candidate binds the kernel effective UID, hashed `/etc/machine-id`, resolved
configuration digest and actual source/artifact/inventory bytes. A machine ID
is local metadata, not hardware attestation or proof against cloned hosts. Source
and artifact hashes are streamed with finite budgets (64/256 MiB), inventory and
readback at most 16 MiB. JSON intake uses a bounded regular-file reader; the core
additionally limits the typed plan to 16 KiB, 16 steps, one-hour consent and a
300-second overall dispatch budget. Unsupported host identity/platform fails
explicitly; there is no environment override that claims another operator/host.

Approval prints the complete candidate and requires the exact plan SHA256 from
an operator terminal. Nonterminal input or a wrong answer records no consent.
The operator must review the source/artifact and target meaning; confirmation is
local consent and does not establish external execution authorization.

The ledger's existing absolute parent must be physical, current-user-owned and
0700. Ancestors must be root/current-user-owned; shared writable ancestors
require the sticky bit. DB and sidecars must be owned single-link regular 0600 files. No mode repair
or adoption of shared/foreign state occurs. Product `plan` and `record` now use
the same private ledger constructor, preserving their existing reconciliation
rows. Legacy local files must be inspected and deliberately secured by their
operator before adoption; do not change a running/foreign journal automatically.
Private maintenance host admission is Linux/WSL; source planning/renderer types
remain independently reusable. No real key store is opened.

## Reserve once, then independently reconcile

```sh
cargo +1.97 run --locked -p vpremises-environment-plan -- maintenance-reserve /ABSOLUTE/PRIVATE/ledger.sqlite3 CANDIDATE.json RESOLVED.json SOURCE ARTIFACT INVENTORY PREFLIGHT_STATE > RESERVATION.json
cargo +1.97 run --locked -p vpremises-environment-plan -- maintenance-status /ABSOLUTE/PRIVATE/ledger.sqlite3 CANDIDATE.json
cargo +1.97 run --locked -p vpremises-environment-plan -- maintenance-reconcile /ABSOLUTE/PRIVATE/ledger.sqlite3 CANDIDATE.json RESERVATION.json INDEPENDENT_READBACK_STATE
```

Reservation rehashes current inputs and takes a fresh precondition from the
supplied local file. It commits `unknown` **before** returning. A lost stdout,
crash or failed hypothetical dispatch never makes the consumed step reservable
again. Retrieve the exact reservation from `maintenance-status` if output was
lost; do not rerun reserve. Unknown also blocks renamed plans for that same
host/environment in the same journal. Never copy/reset/delete the journal as a
retry technique; this local owner boundary is not a global exactly-once guarantee.

The CLI hashes supplied readback bytes and uses its own current clock. Such a
file is a synthetic/local observation, not authenticated cloud/host evidence.
After-state readback advances one step, before-state readback terminates failed
without retry, and any third state stays unknown. Each resolved step retains a
receipt. Observing desired state does not establish which process caused it.

Stop and revoke remain available using the retained candidate, even if current
source/artifact files are no longer available:

```sh
cargo +1.97 run --locked -p vpremises-environment-plan -- maintenance-stop /ABSOLUTE/PRIVATE/ledger.sqlite3 CANDIDATE.json
cargo +1.97 run --locked -p vpremises-environment-plan -- maintenance-revoke /ABSOLUTE/PRIVATE/ledger.sqlite3 CANDIDATE.json
```

They stop remaining reservations; they cannot undo or terminate an external
effect. In-flight unknown remains unknown and can be independently reconciled
after expiry/revocation. Status/control/readback bind the retained candidate and
actual local owner/host without claiming current source hashes are reobserved.
Real process termination/revocation belongs to the accepted host enforcer.

## Reproducible acceptance

```sh
python3 playground/environment-planning/verify.py
```

The existing owner playground now exercises the actual compiled CLI, standard
environment resolve/render/reconciliation and finite maintenance in one temporary
private ledger. Tests use synthetic source/artifact/inventory/state and terminal
confirmation, refusal of nonterminal/wrong consent, artifact drift, duplicate
approval/reservation, renamed unknown retry, readback ambiguity/substitution,
revocation and complete receipt history. Only the synthetic fixture writes its
own before/after state; the CLI never applies an external operation. No account,
provider, timer or real custody setup is required. Full owner CI also tests the
adopted core and product workspace; formatting alone is not integration evidence.
