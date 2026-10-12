# Linux/WSL local use

Select an explicitly mounted directory and an opaque environment identifier.
Use [the audit setup](audit.md) to configure local Zixcel and Crowsi collectors.

```sh
./vpremises-security doctor
./vpremises-security observe /private/observer.json
./vpremises-security report /private/observer.json workstation --jsonl
./vpremises-security audit /private/audit.json workstation --jsonl
```

`observe` reads only metadata and returns aggregate counts. `report` adds time,
configuration identity and unsupported-check coverage. `audit` explicitly permits
content inspection through the configured local adapter and runs configured Crowsi
checks. External configuration holds all paths, definitions, baselines and pins.

Relative roots resolve from the configuration directory. Absolute roots can select
mounts such as `/mnt/c/Inspection/Selected`. Filesystem roots, traversal, linked
roots, unreadable entries and exhausted traversal limits cannot receive a pass.

Exit codes are 0 (scoped success), 1 (invalid invocation/configuration),
2 (incomplete coverage) and 3 (findings). Always capture stdout. Missing collectors
keep `audit` incomplete; `report` without collectors always returns 2.

## Scheduling

Use a systemd timer or cron to invoke the same one-shot command with absolute paths.
No scheduler, service or permissions are installed automatically. Use a least-privilege
account, `umask 077`, private output paths and a new output filename for each run.
Define retention externally and capture reports even for exit codes 2 or 3.
A stopped WSL distribution has no fresh observation; do not infer a successful run.

The tool runs only on Linux/WSL. Mounted Windows files can be inspected, but the
Windows host's network, firewall, services, credentials and ACL state are not covered.
