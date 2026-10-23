"""Exercise operator commands using synthetic fixtures and a disposable ledger."""
import json
import pathlib
import subprocess
import tempfile
from finite_maintenance import verify as verify_finite_maintenance

ROOT = pathlib.Path(__file__).resolve().parents[2]
FIXTURES = pathlib.Path(__file__).resolve().parent


def command(binary, *arguments):
    result = subprocess.run(
        ["cargo", "+1.97", "run", "--locked", "-q", "-p",
         "vpremises-environment-plan", "--bin", binary, "--", *map(str, arguments)],
        cwd=ROOT, text=True, capture_output=True, check=True,
    )
    return json.loads(result.stdout) if result.stdout.strip() else None


with tempfile.TemporaryDirectory(prefix="vpremises-plan-fixture-") as temporary:
    directory = pathlib.Path(temporary)
    resolved = directory / "resolved.json"
    ledger = directory / "ledger.sqlite3"
    receipt = directory / "receipt.json"
    rendered = directory / "main.tf.json"
    command("vpremises-environment-plan", "resolve", FIXTURES / "profiles",
            FIXTURES / "service.toml", FIXTURES / "environment.toml", resolved)
    command("incus-renderer", "render", resolved, rendered)
    assert json.loads(rendered.read_text())["resource"]["incus_instance"]["managed"]["name"] == "fixture-primary"
    assert command("vpremises-environment-plan", "plan", resolved, ledger)["action"] == "apply"
    command("vpremises-environment-plan", "make-receipt", resolved, "incus",
            "synthetic-001", "applied", "true", "2026-07-28T00:00:00Z", receipt)
    assert command("vpremises-environment-plan", "record", resolved, ledger, receipt)["recorded"]
    assert not command("vpremises-environment-plan", "record", resolved, ledger, receipt)["recorded"]
    assert command("vpremises-environment-plan", "plan", resolved, ledger)["action"] == "no-op"
    verify_finite_maintenance(ROOT, directory, resolved)
print("Synthetic resolve/render/plan/receipt/replay/no-op passed; no provider effects")
