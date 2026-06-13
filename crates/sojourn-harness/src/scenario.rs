//! Scenario scripts (RON) and the headless driver (FR-CORE-703, FR-SIM-007):
//! seed + configuration + tick-stamped commands + checkpoints. The driver is the
//! single way every suite command runs a scenario, so stepping-pattern variation
//! is a first-class parameter (FR-CORE-204 verification).

use crate::mutation::Mutation;
use crate::synthetic::{SyntheticConfig, SyntheticModule, toy::ToyModule};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sojourn_core::{
    Command, DataSet, RunConfig, RunMode, SimCore, SimModule, StateFingerprint, StepRequest,
    StopReason,
};
use std::collections::BTreeMap;
use std::path::Path;

/// A command at a tick: a kernel command, an astro command, or a world command
/// (exactly one).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedCommand {
    /// Tick at which the driver submits it.
    pub tick: u64,
    /// A kernel command.
    #[serde(default)]
    pub command: Option<Command>,
    /// An astro command (wrapped into `Command::ModulePayload` at submit).
    #[serde(default)]
    pub astro: Option<sojourn_astro::AstroCommand>,
    /// A world command (wrapped into `Command::ModulePayload` at submit).
    #[serde(default)]
    pub world: Option<sojourn_world::WorldCommand>,
    /// A research command (wrapped into `Command::ModulePayload` at submit).
    #[serde(default)]
    pub research: Option<sojourn_research::ResearchCommand>,
    /// A vehicle command (wrapped into `Command::ModulePayload` at submit).
    #[serde(default)]
    pub vehicle: Option<sojourn_vehicle::VehicleCommand>,
}

impl TimedCommand {
    /// The kernel command this entry submits.
    pub fn to_kernel(&self) -> Result<Command> {
        match (
            &self.command,
            &self.astro,
            &self.world,
            &self.research,
            &self.vehicle,
        ) {
            (Some(c), None, None, None, None) => Ok(c.clone()),
            (None, Some(a), None, None, None) => Ok(sojourn_astro::astro_payload(a)),
            (None, None, Some(w), None, None) => Ok(sojourn_world::world_payload(w)),
            (None, None, None, Some(r), None) => Ok(sojourn_research::research_payload(r)),
            (None, None, None, None, Some(v)) => Ok(sojourn_vehicle::vehicle_payload(v)),
            _ => bail!(
                "each scenario command needs exactly one of `command`, `astro`, `world`, `research` or `vehicle`"
            ),
        }
    }
}

/// A scenario script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    /// Name (reports).
    pub name: String,
    /// Master seed.
    pub seed: u64,
    /// Horizon (25/50/100).
    pub horizon_years: u16,
    /// Save-anywhere or ironman.
    pub run_mode: RunMode,
    /// Install the synthetic module with this configuration.
    pub synthetic: Option<SyntheticConfig>,
    /// Install the toy reference module.
    #[serde(default)]
    pub toy: bool,
    /// Install the astrodynamics module (loads `data/astro`).
    #[serde(default)]
    pub astro: bool,
    /// Install the FA-03 world stack: the astro propagator on the real catalogue
    /// (`data/world`) plus the world module (belief/sites/locations/prospecting).
    #[serde(default)]
    pub world: bool,
    /// Install the FA-05 research module (loads `data/research` + `data/tech`).
    #[serde(default)]
    pub research: bool,
    /// Install the FA-04 vehicle module (loads `data/vehicle`).
    #[serde(default)]
    pub vehicle: bool,
    /// Scripted commands (sorted by tick; ties in listed order).
    pub commands: Vec<TimedCommand>,
    /// Fingerprint checkpoints (ticks).
    pub checkpoints: Vec<u64>,
    /// Drive until this tick.
    pub until_tick: u64,
    /// Optional golden fingerprints (hex), parallel to `checkpoints`.
    #[serde(default)]
    pub expected: Option<Vec<String>>,
}

impl Scenario {
    /// Load and validate a scenario file.
    pub fn load(path: &Path) -> Result<Scenario> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading scenario {}", path.display()))?;
        let s: Scenario =
            ron::from_str(&text).with_context(|| format!("parsing scenario {}", path.display()))?;
        Ok(s)
    }

    /// Build the module set this scenario declares.
    pub fn modules(&self, mutation: Option<Mutation>) -> Result<Vec<Box<dyn SimModule>>> {
        let mut mods: Vec<Box<dyn SimModule>> = Vec::new();
        if let Some(cfg) = &self.synthetic {
            mods.push(Box::new(SyntheticModule {
                cfg: cfg.clone(),
                mutation,
            }));
        }
        if self.toy {
            mods.push(Box::new(ToyModule));
        }
        if self.astro {
            let module = sojourn_astro::AstroModule::load(Path::new("data/astro"))
                .map_err(|e| anyhow::anyhow!("loading astro data: {e}"))?;
            mods.push(Box::new(module));
        }
        if self.world {
            let astro = sojourn_astro::AstroModule::load(Path::new("data/world"))
                .map_err(|e| anyhow::anyhow!("loading astro on real catalogue: {e}"))?;
            let world = sojourn_world::WorldModule::load(Path::new("data/world"))
                .map_err(|e| anyhow::anyhow!("loading world data: {e}"))?;
            mods.push(Box::new(astro));
            mods.push(Box::new(world));
        }
        if self.research {
            let module = sojourn_research::ResearchModule::load(Path::new("data"))
                .map_err(|e| anyhow::anyhow!("loading research data: {e}"))?;
            mods.push(Box::new(module));
        }
        if self.vehicle {
            let module = sojourn_vehicle::VehicleModule::load(Path::new("data/vehicle"))
                .map_err(|e| anyhow::anyhow!("loading vehicle data: {e}"))?;
            mods.push(Box::new(module));
        }
        Ok(mods)
    }

    /// Run configuration.
    pub fn run_config(&self) -> RunConfig {
        RunConfig {
            master_seed: self.seed,
            horizon_years: self.horizon_years,
            run_mode: self.run_mode,
            difficulty_inputs: BTreeMap::new(),
        }
    }

    /// Create a core for this scenario.
    pub fn create(
        &self,
        data: &DataSet,
        run_dir: Option<&Path>,
        mutation: Option<Mutation>,
    ) -> Result<SimCore> {
        let core = match run_dir {
            Some(dir) => SimCore::create_persistent(
                self.run_config(),
                data.clone(),
                self.modules(mutation)?,
                dir,
            )?,
            None => SimCore::create(self.run_config(), data.clone(), self.modules(mutation)?)?,
        };
        Ok(core)
    }
}

/// How the driver steps (the deliberately varied dimension of the double-run
/// check: outcomes must be identical whatever the pattern — FR-CORE-204).
#[derive(Debug, Clone, Copy)]
pub enum StepPattern {
    /// One stride per stop point.
    MaxStride,
    /// Fixed odd-sized chunks.
    Chunk(u64),
}

/// Driver output.
#[derive(Debug, Clone)]
pub struct DriveOutcome {
    /// Fingerprints at the scenario checkpoints (tick, fingerprint).
    pub checkpoints: Vec<(u64, StateFingerprint)>,
    /// Final tick.
    pub final_tick: u64,
    /// Interrupts auto-acknowledged along the way.
    pub acks: u64,
}

/// Drive a core through a scenario: submit commands at their ticks, record
/// checkpoint fingerprints, auto-acknowledge interrupts (the acknowledgements
/// are journaled commands, so replay reproduces the same pattern), and stop at
/// `stop_at` (or the scenario's `until_tick`).
pub fn drive(
    core: &mut SimCore,
    scenario: &Scenario,
    stop_at: Option<u64>,
    pattern: StepPattern,
) -> Result<DriveOutcome> {
    drive_resumed(core, scenario, stop_at, pattern, None)
}

/// As [`drive`], but for a core resumed from a save taken at `resume_after`:
/// scenario commands with `tick <= resume_after` were already applied before the
/// save and must not be re-submitted.
pub fn drive_resumed(
    core: &mut SimCore,
    scenario: &Scenario,
    stop_at: Option<u64>,
    pattern: StepPattern,
    resume_after: Option<u64>,
) -> Result<DriveOutcome> {
    let until = stop_at.unwrap_or(scenario.until_tick);
    let start = core.status().tick;
    let lower = resume_after.map_or(start, |t| t + 1);
    let mut commands: Vec<&TimedCommand> = scenario
        .commands
        .iter()
        .filter(|c| c.tick >= lower && c.tick <= until)
        .collect();
    commands.sort_by_key(|c| c.tick); // stable: ties keep listed order
    // Checkpoints at-or-before the start were already passed (a resumed run does
    // not re-record them); record those strictly after the starting tick.
    let mut checkpoints: Vec<u64> = scenario
        .checkpoints
        .iter()
        .copied()
        .filter(|&t| t <= until && (t > start || (start == 0 && t == 0)))
        .collect();
    checkpoints.sort_unstable();

    let mut out = DriveOutcome {
        checkpoints: Vec::new(),
        final_tick: 0,
        acks: 0,
    };
    let mut cmd_i = 0usize;
    let mut cp_i = 0usize;

    loop {
        let now = core.status().tick;

        // Submit all commands due at this tick, then apply them with a zero-step.
        let mut submitted = false;
        while cmd_i < commands.len() && commands[cmd_i].tick == now {
            core.submit(commands[cmd_i].to_kernel()?)?;
            cmd_i += 1;
            submitted = true;
        }
        if submitted {
            let r = core.step(StepRequest::Ticks(0))?;
            if let StopReason::Interrupted(ids) = r.stopped {
                out.acks += ack_all(core, &ids)?;
            }
        }
        if cmd_i < commands.len() && commands[cmd_i].tick < now {
            bail!(
                "scenario '{}': command at tick {} was overtaken (now at {now}); \
                 commands must land on reachable ticks",
                scenario.name,
                commands[cmd_i].tick
            );
        }

        // Record checkpoints due at this tick.
        while cp_i < checkpoints.len() && checkpoints[cp_i] == now {
            out.checkpoints.push((now, core.fingerprint()?));
            cp_i += 1;
        }

        if now >= until {
            break;
        }

        // Next stop: nearest of (next command, next checkpoint, until).
        let mut next_stop = until;
        if cmd_i < commands.len() {
            next_stop = next_stop.min(commands[cmd_i].tick);
        }
        if cp_i < checkpoints.len() {
            next_stop = next_stop.min(checkpoints[cp_i]);
        }

        let stride = match pattern {
            StepPattern::MaxStride => next_stop - now,
            StepPattern::Chunk(c) => (next_stop - now).min(c.max(1)),
        };
        let r = core.step(StepRequest::Ticks(stride))?;
        match r.stopped {
            StopReason::Completed => {}
            StopReason::Interrupted(ids) => {
                out.acks += ack_all(core, &ids)?;
            }
            StopReason::HorizonReached => {
                // Unattended scenarios continue into sandbox only if the script
                // says so via an explicit ContinueSandbox command; otherwise stop.
                if cmd_i < commands.len()
                    && matches!(commands[cmd_i].command, Some(Command::ContinueSandbox))
                {
                    continue;
                }
                break;
            }
        }
    }

    out.final_tick = core.status().tick;

    // Golden fingerprints, if the scenario carries them.
    if let Some(expected) = &scenario.expected {
        for ((tick, fp), want) in out.checkpoints.iter().zip(expected) {
            if &fp.hex() != want {
                bail!(
                    "scenario '{}': checkpoint at tick {tick} fingerprint {} ≠ golden {want}",
                    scenario.name,
                    fp.hex()
                );
            }
        }
    }
    Ok(out)
}

fn ack_all(core: &mut SimCore, ids: &[u64]) -> Result<u64> {
    for id in ids {
        core.acknowledge(*id)?;
    }
    // Apply the acknowledgements (and surface any follow-on interrupts).
    let r = core.step(StepRequest::Ticks(0))?;
    let mut acked = ids.len() as u64;
    if let StopReason::Interrupted(more) = r.stopped {
        acked += ack_all(core, &more)?;
    }
    Ok(acked)
}

/// Read every event in the run (paged) — for event-log identity comparison.
pub fn full_event_log(core: &SimCore) -> Result<Vec<sojourn_core::EventRecord>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    loop {
        let page = core.events(&sojourn_core::EventFilter {
            offset,
            limit: 4096,
            ..Default::default()
        })?;
        let n = page.events.len() as u64;
        all.extend(page.events);
        if n < 4096 {
            return Ok(all);
        }
        offset += n;
    }
}
