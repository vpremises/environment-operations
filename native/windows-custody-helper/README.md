# vpremises-windows-custody-helper

Serve one bounded credential or authorized signing request using Windows
CurrentUser DPAPI and CNG. The caller supplies a pinned helper identity;
operation-only requests require exact signed authorization and current device status.

This is a Windows executable distributed as a GitHub release asset. The client
SDK is the separate `crowsi-windows-custody-provider` Cargo library. Running this
helper on Linux exits with an unsupported-backend error; there is no plaintext
or shell fallback. A directory mounted in WSL does not supply Windows DPAPI.

```sh
cargo build --locked --release -p vpremises-windows-custody-helper --target x86_64-pc-windows-gnu
```

The command above requires a Windows GNU toolchain and linker. The GitHub release
ZIP is built for Windows x64 with MSVC and requires the Microsoft Visual C++
x64 runtime: the executable imports `VCRUNTIME140.dll`. It must run on Windows,
not as a Linux executable inside WSL. A WSL client can invoke the pinned Windows
helper through Windows interoperability when the host runtime is installed.

Each new release also publishes `windows-custody-manifest.json`, which records
the source commit, helper/provider versions, explicit MSVC target, stdio
contracts, runtime prerequisite, and ZIP/executable SHA-256 digests. Packaging
checks that the ZIP contains the exact helper tested on Windows. Publication
revalidates the descriptor against the checked-out release source and archive.
The descriptor is a source/artifact receipt, not a signed publisher attestation
or proof of full WSL/PA/authority/CNG acceptance. Earlier releases may lack it.

Windows CI also
builds and exercises the helper with isolated synthetic CurrentUser data. Never
send a production credential to a fixture or use a unit test as live-account acceptance.
See the [product README](../../README.md) for composition and private configuration.
