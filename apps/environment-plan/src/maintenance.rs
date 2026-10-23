mod input;
use input::{Candidate, Draft, candidate, confirm, context, local_identity, now, observation};
use vpremises_environment_deployment::{
    Error, Ledger, MaintenanceContext, MaintenancePlan, MaintenanceReservation, Result,
    maintenance_plan_digest,
};

pub fn run(args: &[String]) -> Result<()> {
    match args {
        [command, draft, resolved, source, artifact, inventory]
            if command == "maintenance-plan" =>
        {
            create_plan(draft, resolved, source, artifact, inventory)?;
        }
        [command, ledger, file, resolved, source, artifact, inventory]
            if command == "maintenance-approve" =>
        {
            let candidate = candidate(file)?;
            let observed = context(resolved, source, artifact, inventory)?;
            if observed != candidate.plan.context {
                return Err(Error::new("maintenance current input mismatch"));
            }
            confirm(&candidate)?;
            let digest = Ledger::open_private(ledger)?.approve_maintenance(
                &candidate.plan,
                &observed,
                now()?,
            )?;
            crate::print_json(
                &serde_json::json!({"state":"local-consent-recorded","plan_sha256":digest,"execution_authorized":false}),
            )?;
        }
        [
            command,
            ledger,
            file,
            resolved,
            source,
            artifact,
            inventory,
            state,
        ] if command == "maintenance-reserve" => {
            let candidate = candidate(file)?;
            let observed = context(resolved, source, artifact, inventory)?;
            let now = now()?;
            let preflight = observation(state, now)?;
            let reservation = Ledger::open_private(ledger)?.reserve_maintenance(
                &candidate.plan.id,
                &candidate.plan_sha256,
                &observed,
                &preflight,
                now,
            )?;
            crate::print_json(&reservation)?;
        }
        [command, ledger, file, reservation, state] if command == "maintenance-reconcile" => {
            let candidate = candidate(file)?;
            let reservation: MaintenanceReservation = crate::read_json(reservation)?;
            if reservation.plan_id != candidate.plan.id
                || reservation.plan_sha256 != candidate.plan_sha256
            {
                return Err(Error::new(
                    "maintenance reservation is for another candidate",
                ));
            }
            let now = now()?;
            let observed = observation(state, now)?;
            crate::print_json(&Ledger::open_private(ledger)?.reconcile_maintenance(
                &reservation,
                &owned_context(&candidate)?,
                &observed,
                now,
            )?)?;
        }
        [command, ledger, file]
            if [
                "maintenance-status",
                "maintenance-stop",
                "maintenance-revoke",
            ]
            .contains(&command.as_str()) =>
        {
            control(command, ledger, file)?;
        }
        _ => {
            return Err(Error::new(
                "usage: maintenance-plan DRAFT RESOLVED SOURCE ARTIFACT INVENTORY | maintenance-approve LEDGER CANDIDATE RESOLVED SOURCE ARTIFACT INVENTORY | maintenance-reserve LEDGER CANDIDATE RESOLVED SOURCE ARTIFACT INVENTORY PREFLIGHT | maintenance-reconcile LEDGER CANDIDATE RESERVATION READBACK | maintenance-status|maintenance-stop|maintenance-revoke LEDGER CANDIDATE",
            ));
        }
    }
    Ok(())
}

fn owned_context(candidate: &Candidate) -> Result<MaintenanceContext> {
    let mut context = candidate.plan.context.clone();
    (context.owner, context.host) = local_identity()?;
    Ok(context)
}

fn create_plan(
    draft: &str,
    resolved: &str,
    source: &str,
    artifact: &str,
    inventory: &str,
) -> Result<()> {
    let draft: Draft = crate::read_json(draft)?;
    let issued_at = now()?;
    let plan = MaintenancePlan {
        schema: "vpremises://maintenance/finite-plan/v1".into(),
        id: draft.id,
        environment_id: crate::read_json::<vpremises_environment_deployment::ResolvedEnvironment>(
            resolved,
        )?
        .environment
        .environment_id,
        context: context(resolved, source, artifact, inventory)?,
        issued_at,
        expires_at: issued_at
            .checked_add(i64::from(draft.valid_seconds))
            .ok_or_else(|| Error::new("maintenance validity overflow"))?,
        max_runtime_seconds: draft.max_runtime_seconds,
        steps: draft.steps,
    };
    let plan_sha256 = maintenance_plan_digest(&plan)?;
    crate::print_json(&Candidate { plan, plan_sha256 })?;
    Ok(())
}

fn control(command: &str, ledger: &str, file: &str) -> Result<()> {
    let candidate = candidate(file)?;
    let context = owned_context(&candidate)?;
    let now = now()?;
    let mut ledger = Ledger::open_private(ledger)?;
    let status = match command {
        "maintenance-stop" => {
            ledger.stop_maintenance(&candidate.plan.id, &candidate.plan_sha256, &context, now)?
        }
        "maintenance-revoke" => {
            ledger.revoke_maintenance(&candidate.plan.id, &candidate.plan_sha256, &context, now)?
        }
        _ => {
            ledger.maintenance_status(&candidate.plan.id, &candidate.plan_sha256, &context, now)?
        }
    };
    crate::print_json(&status)?;
    Ok(())
}
