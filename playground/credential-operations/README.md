# vpremises-credential-examples

Explicit synthetic examples for the credential broker library. This package is
not published to crates.io and is not an installed credential service.

```sh
cargo run --locked -p vpremises-credential-examples --bin crowsi-credential-demo
```

The demo generates ephemeral in-memory material and prints metadata only.
The default `crowsi-credential` executable rejects operations without an
authenticated IPC adapter. For configured enrollment and credential operations,
use the product's `crowsi-credential-agent`; a sample status is never readiness
proof for a real account or an installed helper.
