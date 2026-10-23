"""No-network refusal checks for the product/owner distribution boundary."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

PRODUCT = Path(__file__).resolve().parents[1]


class DetectorComposition(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.owner = self.root / 'owner'
        self.stage = self.root / 'stage'
        self.runtime = self.root / 'runtime'
        for directory in [self.owner / 'tests', self.owner / 'scripts', self.stage,
                          self.runtime, self.root / 'scripts']:
            directory.mkdir(parents=True)
        shutil.copyfile(PRODUCT / 'scripts/provision-detector.sh', self.root / 'scripts/provision-detector.sh')
        shutil.copyfile(PRODUCT / 'scripts/collectors.json', self.root / 'scripts/collectors.json')
        pin = json.loads((PRODUCT / 'scripts/collectors.json').read_text())['gitleaks']
        self.pin = dict(url=pin['url'], archive_sha256=pin['archive_sha256'],
                        executable_sha256=pin['binary_sha256'], source=pin['source'], build=pin['build'])
        self.write_pin()
        # This test does not emulate successful trust or real detector execution.
        (self.owner / 'scripts/fetch-published-detector.sh').write_text(
            'set -eu\ntouch "$RUNNER_TEMP/fetch-called"\nmkdir "$1"\nprintf "{}" > "$1/provenance.json"\n')
        (self.owner / 'scripts/audit-published-detector.sh').write_text(
            'set -eu\nmkdir "$1"\nprintf \'{"finding":{"osv":"synthetic-rejected","trace":[]}}\\n\' > "$1/vulnerabilities.jsonl"\nexit 17\n')

    def write_pin(self):
        (self.owner / 'tests/gitleaks-pin.json').write_text(json.dumps(self.pin))

    def run_composition(self):
        return subprocess.run(['bash', 'scripts/provision-detector.sh', str(self.owner), str(self.stage)],
                              cwd=self.root, env={**os.environ, 'RUNNER_TEMP': str(self.runtime),
                              'CARGO_TARGET_DIR': str(self.root / 'target')},
                              capture_output=True, text=True, timeout=10)

    def test_each_substituted_owner_binding_stops_before_fetch(self):
        for field in self.pin:
            with self.subTest(field=field):
                original = self.pin[field]
                self.pin[field] = 'substituted'
                self.write_pin()
                result = self.run_composition()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('pins disagree', result.stderr)
                self.assertFalse((self.runtime / 'fetch-called').exists())
                self.pin[field] = original
        self.write_pin()

    def test_rejected_audit_preserves_findings_and_never_provisions(self):
        result = self.run_composition()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('synthetic-rejected', result.stderr)
        self.assertTrue((self.runtime / 'inspector-detector-evidence/audit/vulnerabilities.jsonl').is_file())
        self.assertFalse((self.stage / 'engine').exists())
        self.assertFalse((self.stage / 'tools/gitleaks').exists())


if __name__ == '__main__':
    unittest.main()
