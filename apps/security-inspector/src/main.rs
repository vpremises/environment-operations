//! vPremises command entrypoint keeps all behavior within the local CLI adapter.

#![forbid(unsafe_code)]

mod cli;

fn main() -> std::process::ExitCode {
    cli::main()
}
