use std::{env, fs};

use vpremises_incus_driver::{ResolvedEnvironment, render};

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if let Err(error) = run(&arguments) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run(arguments: &[String]) -> Result<(), String> {
    let [command, input, output] = arguments else {
        return Err("usage: vpremises-incus-driver render <resolved.json> <main.tf.json>".into());
    };
    if command != "render" {
        return Err("only the render command is supported".into());
    }
    let bytes = fs::read(input).map_err(|error| error.to_string())?;
    let resolved: ResolvedEnvironment =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let rendered = render(&resolved)?;
    fs::write(
        output,
        serde_json::to_vec_pretty(&rendered).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}
