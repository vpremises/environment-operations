# vpremises-security interface reference

Use the [usage guide](getting-started.md) for the first steps. This reference preserves the current interface details and operational limits. Run command examples from the repository root, after preparing the exact declared dependencies and registered configuration.

## Mounted SharePoint report acquisition

`acquire-mounted-sharepoint-report` is a separate, narrowly bounded command.
It does not change the behavior or policy of `observe`. The command reads one
relative JSON file below one explicitly allowlisted mounted root, verifies that
the file satisfies the closed Department Daily Report contract, including:

```json
{
  "schema": "estate://operations/department-daily-report/v1"
}
```

Every required field is parsed and unknown fields are rejected; a matching
`schema` string alone is not sufficient. The closed request also declares the
expected record mode, reporting date, headquarters, department, reporting
team, owner, classification, and customer-data flag. Acquisition fails unless
every value matches the report body, in addition to requiring `artifact_id` to
equal `operational_report_id`.

It rejects absolute and parent-relative file references, non-JSON files,
symbolic links in every path component, the filesystem root, non-regular
files, unknown request fields, and reports larger than both the request limit
and the hard 4 MiB ceiling. It performs no network operation and does not
verify SharePoint membership or sync freshness; the operator is responsible
for mounting the approved library at the allowlisted root.

On Linux, acquisition opens each component from a held parent-directory
descriptor through `/proc/self/fd` with `O_DIRECTORY` and `O_NOFOLLOW`, then
opens the report with `O_NOFOLLOW`. This keeps a rename or symlink swap between
inspection and open from redirecting the read outside the allowlisted root.
Platforms without an equivalent directory-anchored no-follow boundary fail
closed for this acquisition command.

Mounted roots are portable references rather than committed host paths.
`allowlisted_root.base` is a closed choice of `workspace-root` or
`request-directory`, and `relative_path` must remain below the base. The CLI
resolves `workspace-root` by walking from the request file to a regular
`source-foundation.toml`.

The successful receipt contains only opaque request, receipt, correlation,
root, and artifact identifiers plus the report schema ID, media type, byte
size, and SHA-256 digest. It never contains the mounted path, relative path,
report body, filename, or a credential, and always reports
`external_actions: false`.

The request and receipt use closed, versioned contracts:

- [`vpremises.mounted-sharepoint-report-request.v1.schema.json`](../schemas/vpremises.mounted-sharepoint-report-request.v1.schema.json)
- [`vpremises.mounted-sharepoint-report-receipt.v1.schema.json`](../schemas/vpremises.mounted-sharepoint-report-receipt.v1.schema.json)

The bundled request resolves `fixtures` relative to its own request file. To
run it:

```bash
cargo run --offline -- acquire-mounted-sharepoint-report \
  examples/mounted-sharepoint-report.request.json
```

Deployment-specific requests belong in private runtime configuration; keep
their roots workspace-relative and never commit a host-specific absolute path.

## Library use

The observation and mounted-report boundaries are independently usable:

```rust
use vpremises::{observe, ObserverConfig};

let config: ObserverConfig =
    serde_json::from_str(include_str!("examples/observer.repository.json"))?;
let report = observe(&config);
assert_eq!(report.policy.access_mode, "metadata-only");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Library callers of `acquire_mounted_sharepoint_report` must resolve the
declared root base themselves; the function then applies the same bounded
request, no-follow open, closed report, and content-free receipt contracts as
the CLI. Observer configuration files are limited to 1 MiB by the CLI, and
mounted request files to 64 KiB. The package remains local
(`publish = false`) and contains no daemon, network client, provider client,
credential store, or external-action executor.
