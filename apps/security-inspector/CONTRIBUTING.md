# Contributing

Keep changes scoped and explain observable behavior in a pull request. Rust source
files, including tests and examples, must not exceed 120 lines. Split larger modules
by responsibility into named files and directories. Document public types, fields,
functions and errors; explain safety boundaries and non-obvious platform behavior.
Use English documentation and comments. Preserve synthetic test fixtures and tests.

Use the pinned Rust toolchain and an external `CARGO_TARGET_DIR`:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo run --locked --example check-repository
```

Run native Linux/WSL CI before release. Platform-specific no-follow
boundaries must fail closed when unsupported. Never suppress a check or remove a
test to make CI pass. Keep dependency versions locked and preserve license notices.

Do not commit credentials, machine registration, customer data, reports, caches or
build outputs. Report vulnerabilities privately as described in SECURITY.md.
Maintain required reviewers for release preparation. Contributions are licensed
under Apache-2.0; contributors retain their copyrights.
