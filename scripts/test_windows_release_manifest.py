"""Reject archive, architecture, source and contract mismatches before release."""
import importlib.util
import json
import struct
import tempfile
import unittest
import zipfile
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "release", Path(__file__).with_name("windows-release-manifest.py"))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseManifest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.archive = self.root / "windows-custody.zip"
        self.binary = self.root / release.EXECUTABLE
        data = bytearray(96)
        data[:2] = b"MZ"
        struct.pack_into("<I", data, 60, 64)
        data[64:68] = b"PE\0\0"
        struct.pack_into("<H", data, 68, 0x8664)
        self.binary.write_bytes(data)
        self.pack(data)

    def pack(self, data):
        with zipfile.ZipFile(self.archive, "w") as zipped:
            zipped.writestr(release.EXECUTABLE, data)
            zipped.writestr("LICENSE", "synthetic fixture")
            zipped.writestr("NOTICE", "synthetic fixture")

    def create(self, source="a" * 40):
        return release.manifest(release.ROOT, self.archive, source, self.binary)

    def test_archive_binds_current_packages_contract_and_binary(self):
        result = self.create()
        self.assertEqual(result["helper"]["version"], "0.10.0")
        self.assertEqual(result["provider"]["version"], "0.10.2")
        self.assertEqual(result["abi"]["contracts"]["protocol"], "crowsi-windows-custody-v1")
        self.assertEqual(result["abi"]["contracts"]["operation_request_schema"],
                         "crowsi://platform-custody/operation-request/v3")
        self.assertEqual(result["assets"]["executable"]["sha256"],
                         release.digest(release.EXECUTABLE, self.binary.read_bytes())["sha256"])
        self.assertNotIn("full acceptance passed", result["acceptance_scope"])

    def test_archived_binary_must_equal_the_tested_binary(self):
        self.binary.write_bytes(self.binary.read_bytes() + b"changed")
        with self.assertRaisesRegex(ValueError, "tested helper"):
            self.create()

    def test_non_windows_and_other_architectures_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "PE executable"):
            release.windows_x64(b"Linux executable")
        data = bytearray(self.binary.read_bytes())
        struct.pack_into("<H", data, 68, 0xAA64)
        self.pack(data)
        with self.assertRaisesRegex(ValueError, "architecture"):
            self.create()

    def test_abbreviated_source_ids_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "full Git"):
            self.create("a" * 7)

    def test_archive_changes_break_recorded_digest(self):
        before = self.create()
        descriptor = self.root / "manifest.json"
        descriptor.write_text(json.dumps(before))
        release.verify_manifest(release.ROOT, self.archive, "a" * 40, descriptor)
        with zipfile.ZipFile(self.archive, "a") as zipped:
            zipped.writestr("extra.txt", "changed after build")
        after = self.create()
        self.assertNotEqual(before["assets"]["archive"], after["assets"]["archive"])
        with self.assertRaisesRegex(ValueError, "does not match"):
            release.verify_manifest(release.ROOT, self.archive, "a" * 40, descriptor)

    def test_source_and_abi_tampering_are_rejected(self):
        descriptor = self.root / "manifest.json"
        result = self.create()
        descriptor.write_text(json.dumps(result))
        with self.assertRaisesRegex(ValueError, "does not match"):
            release.verify_manifest(release.ROOT, self.archive, "b" * 40, descriptor)
        result["abi"]["contracts"]["operation_request_schema"] = "unreviewed"
        descriptor.write_text(json.dumps(result))
        with self.assertRaisesRegex(ValueError, "does not match"):
            release.verify_manifest(release.ROOT, self.archive, "a" * 40, descriptor)


if __name__ == "__main__":
    unittest.main()
