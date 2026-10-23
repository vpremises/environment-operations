# Synthetic environment planning

From the repository root, run:

```sh
python3 playground/environment-planning/verify.py
```

This playground resolves synthetic product requirements and placement, renders Incus configuration, calculates a plan, records a synthetic receipt and verifies that an identical plan becomes a no-op. All outputs and the ledger are in a temporary directory removed after the test. No provider connection, OpenTofu apply, service installation or timer activation occurs.

The operator commands use the reviewed public `vpremises/environment-deployment` Git revision in Cargo.lock. They do not require a sibling repository, private registry or production profile.

The same verification also runs `finite_maintenance.py` against the compiled
operator CLI: private local consent, one-use reservation, unknown-result refusal,
renamed retry refusal, stop/revoke and exact readback/receipts. The fixture writes
only disposable synthetic state; the CLI starts no maintenance adapter.

A receipt in this playground is synthetic evidence for exercising the ledger; it is not an authenticated production execution receipt. Production execution, signing and authorization require their own established operation path.
