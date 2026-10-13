use std::{env, fs, path::Path};

use vpremises_environment_deployment::{
    Ledger, Receipt, ResolvedEnvironment, Result, desired_digest, load_environment,
    load_profile_directory, load_service, resolve,
};

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if let Err(error) = run(&arguments) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run(arguments: &[String]) -> Result<()> {
    match arguments {
        [command, resolved, binding, inventory, jitter] if command == "render-maintenance" => {
            let desired = read_json::<ResolvedEnvironment>(resolved)?;
            let binding =
                read_json::<vpremises_environment_deployment::MaintenanceBinding>(binding)?;
            let jitter = jitter.parse().map_err(|_| {
                vpremises_environment_deployment::Error::new("invalid maintenance jitter")
            })?;
            print_json(&vpremises_environment_deployment::render_maintenance_timer(
                &desired,
                &binding,
                &fs::read(inventory)?,
                jitter,
            )?)?;
        }
        [command, profiles, service, environment, output] if command == "resolve" => {
            let resolved = resolve(
                &load_profile_directory(profiles)?,
                load_service(service)?,
                load_environment(environment)?,
            )?;
            fs::write(output, serde_json::to_vec_pretty(&resolved)?)?;
        }
        [command, resolved, ledger] if command == "plan" => {
            let desired = read_json::<ResolvedEnvironment>(resolved)?;
            print_json(&Ledger::open(ledger)?.plan(&desired)?)?;
        }
        [command, resolved, ledger, receipt] if command == "record" => {
            let desired = read_json::<ResolvedEnvironment>(resolved)?;
            let receipt = read_json::<Receipt>(receipt)?;
            let recorded = Ledger::open(ledger)?.record(&desired, &receipt)?;
            print_json(&serde_json::json!({ "recorded": recorded }))?;
        }
        [
            command,
            resolved,
            driver,
            operation,
            status,
            changed,
            observed,
            output,
        ] if command == "make-receipt" => {
            let desired = read_json::<ResolvedEnvironment>(resolved)?;
            let receipt = Receipt {
                schema: "vpremises://infrastructure/apply-receipt/v1".into(),
                operation_id: operation.clone(),
                environment_id: desired.environment.environment_id.clone(),
                desired_digest: desired_digest(&desired)?,
                driver: driver.clone(),
                status: status.clone(),
                changed: changed.parse().map_err(|_| {
                    vpremises_environment_deployment::Error::new("invalid changed value")
                })?,
                observed_at: observed.clone(),
            };
            fs::write(output, serde_json::to_vec_pretty(&receipt)?)?;
        }
        _ => {
            return Err(vpremises_environment_deployment::Error::new(
                "usage: vpremises-environment-plan resolve ... | plan ... | make-receipt ... | record ... | render-maintenance RESOLVED BINDING INVENTORY JITTER_SECONDS",
            ));
        }
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
