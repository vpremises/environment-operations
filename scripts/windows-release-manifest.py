"""Bind the standalone Windows archive to reviewed source and protocol versions."""
import argparse
import hashlib
import json
import re
import struct
import subprocess
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-msvc"
EXECUTABLE = "vpremises-windows-custody-helper.exe"
SCHEMA = "vpremises://windows-custody/release-manifest/v1"


def digest(name, data):
    return {"name": name, "size_bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest()}


def windows_x64(data):
    if len(data) < 64 or data[:2] != b"MZ":
        raise ValueError("helper is not a Windows PE executable")
    offset = struct.unpack_from("<I", data, 60)[0]
    if offset + 6 > len(data) or data[offset:offset + 4] != b"PE\0\0":
        raise ValueError("invalid Windows PE header")
    if struct.unpack_from("<H", data, offset + 4)[0] != 0x8664:
        raise ValueError("helper architecture does not match Windows x64")


def manifest(root, archive, source_sha, binary=None):
    if not re.fullmatch(r"[a-f0-9]{40}", source_sha):
        raise ValueError("source SHA must be a full Git commit ID")
    if archive.name != "windows-custody.zip":
        raise ValueError("unexpected standalone archive name")
    with zipfile.ZipFile(archive) as zipped:
        if zipped.namelist().count(EXECUTABLE) != 1:
            raise ValueError("archive must contain exactly one root helper executable")
        for notice in ["LICENSE", "NOTICE"]:
            if zipped.namelist().count(notice) != 1:
                raise ValueError("archive is missing a unique license/notice")
        executable = zipped.read(EXECUTABLE)
    windows_x64(executable)
    if binary is not None and binary.read_bytes() != executable:
        raise ValueError("archive executable differs from the tested helper")
    helper = tomllib.loads((root / "native/windows-custody-helper/Cargo.toml").read_text())
    dependency = helper["dependencies"]["crowsi-windows-custody-provider"]
    revision = dependency.get("rev", "")
    repository = "https://github.com/crowsi-net/credential-custody.git"
    if dependency.get("git") != repository or not re.fullmatch(r"[a-f0-9]{40}", revision):
        raise ValueError("provider must use the approved public repository and exact revision")
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--manifest-path", str(root / "Cargo.toml")],
        text=True))
    def source_manifest(name):
        matches = [p for p in metadata["packages"] if p["name"] == name]
        expected_source = "git+" + repository + "?rev=" + revision + "#" + revision
        if len(matches) != 1 or matches[0].get("source") != expected_source:
            raise ValueError("provider source is not exactly bound to reviewed revision")
        return Path(matches[0]["manifest_path"])
    provider_manifest = source_manifest("crowsi-windows-custody-provider")
    operation_manifest = source_manifest("crowsi-windows-operation-contracts")
    provider = tomllib.loads(provider_manifest.read_text())
    helper_package, provider_package = helper["package"], provider["package"]
    if helper["dependencies"][provider_package["name"]]["version"] != "=" + provider_package["version"]:
        raise ValueError("helper/provider versions are not exactly bound")
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    operation = tomllib.loads(operation_manifest.read_text())["package"]
    if "operation-only" not in provider.get("features", {}).get("default", []):
        raise ValueError("standalone helper no longer enables operation-only contracts by default")
    if provider["dependencies"][operation["name"]]["version"] != "=" + operation["version"]:
        raise ValueError("provider/operation contract versions are not exactly bound")
    for package in [helper_package, provider_package, operation]:
        versions = [p["version"] for p in lock["package"] if p["name"] == package["name"]]
        if versions != [package["version"]]:
            raise ValueError("source and locked package version differ")
    protocol = (provider_manifest.parent / "src/protocol/mod.rs").read_text()
    contracts = {}
    for key in ["PROTOCOL", "REQUEST_SCHEMA", "RESPONSE_SCHEMA", "RUNTIME_SCHEMA"]:
        values = re.findall(r"pub const " + key + r': &str = "([^"]+)";', protocol)
        if len(values) != 1:
            raise ValueError("missing or ambiguous public stdio contract")
        contracts[key.lower()] = values[0]
    operation_source = (operation_manifest.parent / "src/contract.rs").read_text()
    for key in ["OPERATION_REQUEST_SCHEMA", "OPERATION_AUTHORIZATION_SCHEMA", "OPERATION_RESPONSE_SCHEMA"]:
        values = re.findall(r"pub const " + key + r': &str =\s*"([^"]+)";', operation_source)
        if len(values) != 1:
            raise ValueError("missing or ambiguous operation contract")
        contracts[key.lower()] = values[0]
    return {
        "schema": SCHEMA,
        "repository": "https://github.com/vpremises/environment-operations",
        "source_sha": source_sha,
        "provider_source": {"repository": repository, "revision": revision},
        "helper": {"name": helper_package["name"], "version": helper_package["version"]},
        "provider": {"name": provider_package["name"], "version": provider_package["version"]},
        "operation_contracts": {"name": operation["name"], "version": operation["version"]},
        "target": TARGET,
        "abi": {"transport": "bounded anonymous stdio", "contracts": contracts,
                "rust_version": helper_package["rust-version"],
                "windows_runtime": "Microsoft Visual C++ x64 runtime (VCRUNTIME140.dll)",
                "custody_scope": "Windows CurrentUser DPAPI; operation-only CNG signing"},
        "assets": {"archive": digest(archive.name, archive.read_bytes()),
                   "executable": digest(EXECUTABLE, executable)},
        "acceptance_scope": "synthetic helper/CurrentUser tests; not full Windows/WSL acceptance",
    }


def verify_manifest(root, archive, source_sha, descriptor):
    if json.loads(descriptor.read_text()) != manifest(root, archive, source_sha):
        raise ValueError("release manifest does not match source and archive")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["create", "verify"])
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--binary", type=Path)
    args = parser.parse_args()
    if args.command == "create" and args.binary is None:
        parser.error("create requires the tested --binary")
    if args.command == "create":
        expected = manifest(ROOT, args.archive, args.source_sha, args.binary)
        args.manifest.write_text(json.dumps(expected, indent=2) + "\n")
    else:
        verify_manifest(ROOT, args.archive, args.source_sha, args.manifest)
    print("Windows release manifest " + args.command + " succeeded")


if __name__ == "__main__":
    main()
