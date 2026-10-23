"""Disposable real-CLI lifecycle acceptance; no host adapter or provider effect."""
import hashlib
import json
import os
import pathlib
import pty
import subprocess


def verify(root, directory, resolved):
    binary = pathlib.Path(os.environ.get("CARGO_TARGET_DIR", root / "target")) / "debug/vpremises-environment-plan"
    if not binary.is_absolute():
        binary = root / binary
    source = directory / "maintenance-source"
    artifact = directory / "maintenance-artifact"
    inventory = directory / "maintenance-inventory"
    state = directory / "maintenance-state"
    draft = directory / "maintenance-draft.json"
    candidate_file = directory / "maintenance-candidate.json"
    reservation_file = directory / "maintenance-reservation.json"
    ledger = directory / "ledger.sqlite3"  # Reuse the existing reconciliation ledger.
    for path, data in [(source, b"synthetic reviewed source"), (artifact, b"synthetic reviewed artifact"),
                       (inventory, b"synthetic inventory"), (state, b"before")]:
        path.write_bytes(data)
    digest = lambda data: hashlib.sha256(data).hexdigest()
    draft_value = {"id": "synthetic-maintenance", "valid_seconds": 300, "max_runtime_seconds": 60,
                   "steps": [{"id": "first", "operation": "synthetic-update",
                              "before_sha256": digest(b"before"), "after_sha256": digest(b"after-one")},
                             {"id": "second", "operation": "synthetic-checkpoint",
                              "before_sha256": digest(b"after-one"), "after_sha256": digest(b"after-two")}]}
    draft.write_text(json.dumps(draft_value))

    def run(*args, accept=True, terminal_answer=None):
        command = [str(binary), *map(str, args)]
        if terminal_answer is None:
            result = subprocess.run(command, capture_output=True, text=True, timeout=20, stdin=subprocess.DEVNULL)
        else:
            master, slave = pty.openpty()
            try:
                child = subprocess.Popen(command, stdin=slave, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                os.close(slave)
                slave = None
                os.write(master, (terminal_answer + "\n").encode())
                try:
                    stdout, stderr = child.communicate(timeout=20)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.communicate()
                    raise
                result = subprocess.CompletedProcess(command, child.returncode, stdout, stderr)
            finally:
                os.close(master)
                if slave is not None:
                    os.close(slave)
        assert (result.returncode == 0) == accept, (args[0], result.stderr)
        return json.loads(result.stdout) if result.stdout.strip() else None

    inputs = (resolved, source, artifact, inventory)
    candidate = run("maintenance-plan", draft, *inputs)
    candidate_file.write_text(json.dumps(candidate))
    assert candidate["plan"]["context"]["owner"] == f"unix-uid:{os.geteuid()}"
    assert candidate["plan"]["context"]["source_sha256"] == digest(source.read_bytes())
    approve = ("maintenance-approve", ledger, candidate_file, *inputs)
    run(*approve, accept=False)  # A pipe is not owner confirmation.
    run(*approve, accept=False, terminal_answer="wrong digest")
    approved = run(*approve, terminal_answer=candidate["plan_sha256"])
    assert approved["execution_authorized"] is False
    assert (ledger.stat().st_mode & 0o777) == 0o600
    run(*approve, accept=False, terminal_answer=candidate["plan_sha256"])

    reserve = ("maintenance-reserve", ledger, candidate_file, *inputs, state)
    artifact.write_bytes(b"changed artifact")
    run(*reserve, accept=False)
    artifact.write_bytes(b"synthetic reviewed artifact")
    reservation = run(*reserve)
    assert reservation["execution_authorized"] is False
    reservation_file.write_text(json.dumps(reservation))
    run(*reserve, accept=False)  # Unknown, never a retry.
    assert run("maintenance-status", ledger, candidate_file)["phase"] == "unknown"

    # A renamed valid plan cannot bypass an unknown host/environment operation.
    draft_value["id"] = "renamed-synthetic-maintenance"
    draft.write_text(json.dumps(draft_value))
    renamed = run("maintenance-plan", draft, *inputs)
    renamed_file = directory / "renamed-candidate.json"
    renamed_file.write_text(json.dumps(renamed))
    run("maintenance-approve", ledger, renamed_file, *inputs,
        accept=False, terminal_answer=renamed["plan_sha256"])

    # This fixture is the synthetic producer. The CLI never writes target state.
    state.write_bytes(b"unclassified-state")
    readback = ("maintenance-reconcile", ledger, candidate_file, reservation_file, state)
    assert run(*readback)["phase"] == "unknown"
    tampered = dict(reservation, execution_authorized=True)
    reservation_file.write_text(json.dumps(tampered))
    run(*readback, accept=False)
    reservation_file.write_text(json.dumps(reservation))
    state.write_bytes(b"after-one")
    observed = run(*readback)
    assert observed["phase"] == "approved" and observed["next_step"] == 1
    assert len(observed["receipts"]) == 1
    run(*readback, accept=False)

    second = run(*reserve)
    reservation_file.write_text(json.dumps(second))
    stopped = run("maintenance-revoke", ledger, candidate_file)
    assert stopped["phase"] == "unknown" and stopped["revoked"]
    state.write_bytes(b"after-two")
    completed = run(*readback)
    assert completed["phase"] == "completed" and completed["revoked"]
    assert completed["execution_authorized"] is False
    assert len(completed["receipts"]) == 2
    run(*reserve, accept=False)
    # Legacy reconciliation stays in the same physical ledger and keeps working.
    assert run("plan", resolved, ledger)["action"] == "no-op"
    assert source.read_bytes() == b"synthetic reviewed source"
    assert inventory.read_bytes() == b"synthetic inventory"
    print("Synthetic CLI consent/reserve/unknown/revoke/readback/receipt passed; no provider or host activation")
