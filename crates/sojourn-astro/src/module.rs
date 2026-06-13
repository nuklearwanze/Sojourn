//! `SimModule` implementation (T016): the astro slice, the step engine with
//! state-driven tiers, command handling, view publication and serde. Catalogue,
//! engines and config are immutable module data; everything dynamic lives in
//! the slice (single-writer, kernel-serialized).

use crate::bodies::rails::BodyStates;
use crate::bodies::{BodyId, BodyMotion, Catalog, divert};
use crate::craft::{ArcEnd, Craft, CraftId, FlightStatus, GuidanceLaw, MassState, Tier};
use crate::forces;
use crate::maneuver::{
    ActiveBurn, AeroPassPlan, AimPoint, AstroCommand, ManeuverNode, Plan, StationKeeping,
    apply_execution_error, dv_world,
};
use crate::math::{StateVec, Vec3};
use crate::propulsion::{EngineBinding, Engines, delivered_thrust};
use crate::soi::dominant_of;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sojourn_core::{
    CommandOutcome, CoreError, EscalationDecl, InitCtx, ModuleManifest, SimModule, StateSlice,
    StepCtx, Tick, ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Module configuration (DATA, `data/astro/config.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstroConfig {
    /// Coast-tier step (s) — also the module cadence.
    pub coast_step_s: u64,
    /// Encounter-tier step (s).
    pub encounter_step_s: u64,
    /// Burn-tier substep (s).
    pub burn_substep_s: f64,
    /// Accelerations below this integrate at coast tier with thrust (m/s²).
    pub lowthrust_accel_threshold: f64,
    /// Encounter tier within this multiple of the dominant body's radius.
    pub encounter_radius_factor: f64,
    /// Reconciliation divergence threshold (m).
    pub divergence_threshold_m: f64,
    /// Reconciliation cadence (ticks).
    pub reconcile_interval_ticks: u64,
    /// Max simultaneously diverted bodies (FR-ASTRO-107).
    pub diversion_budget: u32,
    /// Execution-error model (FR-ASTRO-304).
    pub exec_error: ExecErrorCfg,
    /// Lambert iteration cap.
    pub lambert_max_iter: u32,
    /// Lambert convergence tolerance.
    pub lambert_tol: f64,
    /// Drag coefficient.
    pub drag_cd: f64,
    /// SRP reflectivity coefficient.
    pub srp_cr: f64,
    /// Solar flux at the reference distance (W/m²).
    pub solar_flux_w_m2: f64,
    /// Flux reference distance (m).
    pub flux_reference_m: f64,
    /// Speed of light (m/s).
    pub light_speed_m_s: f64,
    /// Aero depth limit: periapsis floor (m altitude).
    pub aero_min_periapsis_alt_m: f64,
    /// Provenance.
    pub source: String,
    /// Perturbation switches (FR-ASTRO-102; default all on).
    #[serde(default = "default_true")]
    pub enable_third_body: bool,
    /// J2 switch.
    #[serde(default = "default_true")]
    pub enable_j2: bool,
    /// SRP switch.
    #[serde(default = "default_true")]
    pub enable_srp: bool,
    /// Drag switch.
    #[serde(default = "default_true")]
    pub enable_drag: bool,
}

fn default_true() -> bool {
    true
}

impl AstroConfig {
    /// Load `config.ron` from a directory.
    pub fn load_dir(dir: &Path) -> Result<AstroConfig, String> {
        let path = dir.join("config.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        let cfg: AstroConfig = ron::from_str(&text).map_err(|e| format!("config.ron: {e}"))?;
        if cfg.source.trim().is_empty() {
            return Err("config.ron: empty source".into());
        }
        if cfg.coast_step_s == 0 || cfg.encounter_step_s == 0 || cfg.burn_substep_s <= 0.0 {
            return Err("config.ron: step tiers must be positive".into());
        }
        Ok(cfg)
    }
}

/// Execution-error configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecErrorCfg {
    /// Magnitude sigma (fraction of Δv).
    pub sigma_mag: f64,
    /// Pointing sigma (rad).
    pub sigma_point_rad: f64,
    /// Enabled?
    pub enabled: bool,
    /// Provenance.
    pub source: String,
}

/// The astro slice — everything dynamic, kernel-serialized, single-writer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstroSlice {
    /// Craft store (ordered).
    pub craft: BTreeMap<u64, Craft>,
    /// Manoeuvre nodes.
    pub nodes: BTreeMap<u64, ManeuverNode>,
    /// Committed plans by craft.
    pub plans: BTreeMap<u64, Plan>,
    /// Active finite burns by craft.
    pub burns: BTreeMap<u64, ActiveBurn>,
    /// Station-keeping schedules by craft.
    pub station_keeping: BTreeMap<u64, StationKeeping>,
    /// Planned aero passes by craft.
    pub aero_passes: BTreeMap<u64, AeroPassPlan>,
    /// Small-body motion overrides (FR-ASTRO-107).
    pub motions: BTreeMap<BodyId, BodyMotion>,
    /// Research gates (FA-05 stand-in).
    pub gates: BTreeMap<String, bool>,
    /// Id counters.
    pub next_craft: u64,
    /// Next node id.
    pub next_node: u64,
    /// Last tick this module integrated to.
    pub last_step_tick: u64,
    /// Next reconciliation tick.
    pub next_reconcile_tick: u64,
    /// Escalation flag for the kernel (computed each boundary).
    pub fine_needed: bool,
    /// Catalogue identity guard (saves must reload against the same data).
    pub catalog_hash: [u8; 32],
}

/// The astrodynamics module.
pub struct AstroModule {
    /// Body catalogue (immutable data; FA-03's seat).
    pub catalog: Catalog,
    /// Fixture engines (FA-04's seat).
    pub engines: Engines,
    /// Configuration.
    pub config: AstroConfig,
}

impl AstroModule {
    /// Load catalogue, engines and config from a data directory.
    pub fn load(dir: &Path) -> Result<AstroModule, String> {
        Ok(AstroModule {
            catalog: Catalog::load_dir(dir)?,
            engines: Engines::load_dir(dir)?,
            config: AstroConfig::load_dir(dir)?,
        })
    }

    fn slice<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut AstroSlice {
        slice
            .as_any_mut()
            .downcast_mut::<AstroSlice>()
            .expect("astro slice type")
    }
}

/// Encode an [`AstroCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn astro_payload(cmd: &AstroCommand) -> sojourn_core::Command {
    sojourn_core::Command::ModulePayload {
        module: "astro".into(),
        kind: kind_of(cmd).into(),
        payload: postcard::to_allocvec(cmd).expect("astro command encodes"),
    }
}

fn kind_of(cmd: &AstroCommand) -> &'static str {
    use AstroCommand::*;
    match cmd {
        SpawnCraft { .. } => "spawn-craft",
        DespawnCraft { .. } => "despawn-craft",
        CreateNode { .. } => "create-node",
        EditNode { .. } => "edit-node",
        DeleteNode { .. } => "delete-node",
        CommitPlan { .. } => "commit-plan",
        SetThrottle { .. } => "set-throttle",
        SetGuidanceArc { .. } => "set-guidance-arc",
        ClearGuidanceArc { .. } => "clear-guidance-arc",
        ScheduleStationKeeping { .. } => "schedule-station-keeping",
        CancelStationKeeping { .. } => "cancel-station-keeping",
        PlanAeroPass { .. } => "plan-aero-pass",
        DivertBody { .. } => "divert-body",
        ReRailBody { .. } => "re-rail-body",
        SetResearchGate { .. } => "set-research-gate",
    }
}

impl SimModule for AstroModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "astro".into(),
            owned_slice: "astro/slice".into(),
            publishes: vec!["astro/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "maneuver-node".into(),
                "soi-crossing".into(),
                "impact".into(),
                "atmosphere-entry".into(),
                "plan-invalidated".into(),
                "propellant-exhausted".into(),
                "aero-violation".into(),
                "kernel-diagnostic".into(),
            ],
            subscribes: vec![],
            phase: 0,
            streams: vec!["astro/exec-error".into()],
            cadence_ticks: self.config.coast_step_s,
            escalations: vec![EscalationDecl {
                view: "astro/status".into(),
                field: "fine_needed".into(),
                op: sojourn_core::CmpOp::Eq,
                value: ViewValue::Bool(true),
            }],
        }
    }

    fn init(&self, _ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        Box::new(AstroSlice {
            craft: BTreeMap::new(),
            nodes: BTreeMap::new(),
            plans: BTreeMap::new(),
            burns: BTreeMap::new(),
            station_keeping: BTreeMap::new(),
            aero_passes: BTreeMap::new(),
            motions: BTreeMap::new(),
            gates: BTreeMap::new(),
            next_craft: 0,
            next_node: 0,
            last_step_tick: 0,
            next_reconcile_tick: self.config.reconcile_interval_ticks,
            fine_needed: false,
            catalog_hash: self.catalog.content_hash,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        let s = self.slice(slice);
        let now = ctx.tick().0;
        if now <= s.last_step_tick {
            return;
        }
        let from = s.last_step_tick;
        s.last_step_tick = now;
        let dt_s = (now - from) as f64; // base tick = 1 s (kernel config)

        step_world(self, s, ctx, from, now, dt_s);
        s.fine_needed = compute_fine_needed(self, s, now);
    }

    fn on_command(
        &self,
        slice: &mut dyn StateSlice,
        kind: &str,
        payload: &[u8],
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let s = self.slice(slice);
        let cmd: AstroCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed astro payload (kind '{kind}'): {e}"
                ));
            }
        };
        let outcome = apply_command(self, s, ctx, cmd);
        s.fine_needed = compute_fine_needed(self, s, ctx.tick().0);
        outcome
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<AstroSlice>()
            .expect("astro slice type");
        let active_burns = s.burns.len() as u64;
        let diverted = s
            .motions
            .values()
            .filter(|m| matches!(m, BodyMotion::Diverted { .. }))
            .count() as u64;
        let next_node_tick = s
            .nodes
            .values()
            .filter(|n| n.committed && !n.executed)
            .map(|n| n.epoch_tick)
            .min()
            .unwrap_or(u64::MAX);
        vec![ViewSnapshot {
            id: "astro/status".into(),
            fields: BTreeMap::from([
                (
                    "craft_count".to_string(),
                    ViewValue::U64(s.craft.len() as u64),
                ),
                ("active_burns".to_string(), ViewValue::U64(active_burns)),
                ("diverted_count".to_string(), ViewValue::U64(diverted)),
                ("next_node_tick".to_string(), ViewValue::U64(next_node_tick)),
                ("fine_needed".to_string(), ViewValue::Bool(s.fine_needed)),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<AstroSlice>()
            .expect("astro slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: AstroSlice = postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })?;
        if s.catalog_hash != self.catalog.content_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.catalog_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's astro catalogue differs from the loaded data/astro content; \
                       install the matching astro data version"
                    .into(),
            });
        }
        Ok(Box::new(s))
    }
}

// ----------------------------------------------------------------- stepping

/// Should the kernel run us every tick next window?
fn compute_fine_needed(m: &AstroModule, s: &AstroSlice, now: u64) -> bool {
    if !s.burns.is_empty() {
        return true;
    }
    let horizon = 2 * m.config.coast_step_s;
    // Imminent committed nodes or SK burns.
    if s.nodes
        .values()
        .any(|n| n.committed && !n.executed && n.epoch_tick <= now + horizon)
    {
        return true;
    }
    if s.station_keeping
        .values()
        .any(|sk| sk.next_tick <= now + horizon)
    {
        return true;
    }
    // Craft (or diverted bodies) in encounter/atmosphere regimes.
    let motions = &s.motions;
    for c in s.craft.values() {
        if c.status != FlightStatus::Propagating {
            continue;
        }
        if in_encounter_zone(m, c.dominant, c.state.r) {
            return true;
        }
        // High-accel guidance arcs need fine stepping too.
        if c.guidance.is_some()
            && let Some(def) = c
                .inline_engine
                .as_ref()
                .or_else(|| m.engines.get(&c.engine))
        {
            let accel = def.max_thrust_n / c.mass.total().max(1.0);
            if accel >= m.config.lowthrust_accel_threshold {
                return true;
            }
        }
    }
    for (id, motion) in motions {
        if let BodyMotion::Diverted { dominant, state } = motion {
            let _ = id;
            if in_encounter_zone(m, *dominant, state.r) {
                return true;
            }
        }
    }
    false
}

fn in_encounter_zone(m: &AstroModule, dominant: BodyId, r: Vec3) -> bool {
    let dom = m.catalog.body(dominant).expect("dominant");
    let zone = dom.radius * m.config.encounter_radius_factor;
    let alt_zone = dom
        .atmosphere
        .as_ref()
        .map(|a| dom.radius + a.interface_alt_m * 3.0)
        .unwrap_or(0.0);
    let rn = r.norm();
    rn < zone.max(alt_zone)
}

/// Integrate the world from `from` to `now`.
#[allow(clippy::too_many_lines)]
fn step_world(
    m: &AstroModule,
    s: &mut AstroSlice,
    ctx: &mut StepCtx<'_>,
    from: u64,
    now: u64,
    dt_s: f64,
) {
    let t_mid_ns = ((from + now) / 2) as i64 * 1_000_000_000;
    let t_now_ns = now as i64 * 1_000_000_000;

    // --- Ignitions: committed nodes whose epoch has passed (strictly before
    // `now`, so the kernel's pause at the epoch tick precedes the burn).
    let due: Vec<u64> = s
        .nodes
        .values()
        .filter(|n| n.committed && !n.executed && n.epoch_tick < now)
        .map(|n| n.id)
        .collect();
    for node_id in due {
        let node = s.nodes.get(&node_id).expect("node").clone();
        let craft_id = node.craft.0;
        if s.burns.contains_key(&craft_id) {
            continue; // already burning; next node waits
        }
        let Some(c) = s.craft.get(&craft_id) else {
            continue;
        };
        if c.status != FlightStatus::Propagating {
            continue;
        }
        let dvw = dv_world(node.dv_prn, &c.state);
        let dv_mag = dvw.norm();
        if dv_mag <= 0.0 {
            s.nodes.get_mut(&node_id).expect("node").executed = true;
            continue;
        }
        // Seeded execution error (FR-ASTRO-304): uniform draws from the stream.
        let cfg = m.config.exec_error.clone();
        let rng = ctx.rng("astro/exec-error");
        let mut uniform = || (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let (dir, dv) = apply_execution_error(
            dvw.normalized(),
            dv_mag,
            cfg.sigma_mag,
            cfg.sigma_point_rad,
            cfg.enabled,
            &mut uniform,
        );
        s.burns.insert(
            craft_id,
            ActiveBurn {
                node: node_id,
                direction: dir,
                remaining_dv: dv,
            },
        );
        s.nodes.get_mut(&node_id).expect("node").executed = true;
    }

    // --- Station-keeping burns due in (from, now].
    let sk_due: Vec<u64> = s
        .station_keeping
        .iter()
        .filter(|(_, sk)| sk.next_tick > from && sk.next_tick <= now)
        .map(|(id, _)| *id)
        .collect();
    for craft_id in sk_due {
        apply_station_keeping(m, s, craft_id, t_now_ns);
    }

    // --- Integrate craft.
    let craft_ids: Vec<u64> = s.craft.keys().copied().collect();
    for id in craft_ids {
        integrate_craft(m, s, ctx, id, now, dt_s, t_mid_ns);
    }

    // --- Integrate diverted bodies (gravity only).
    let diverted: Vec<BodyId> = s
        .motions
        .iter()
        .filter(|(_, mo)| matches!(mo, BodyMotion::Diverted { .. }))
        .map(|(id, _)| *id)
        .collect();
    for id in diverted {
        integrate_diverted(m, s, ctx, id, dt_s, t_mid_ns);
    }

    // --- Reconciliation cadence (FR-ASTRO-402): surfaced via diagnostics.
    if now >= s.next_reconcile_tick {
        s.next_reconcile_tick = now + m.config.reconcile_interval_ticks;
        reconcile_all(m, s, ctx, now);
    }
}

fn integrate_craft(
    m: &AstroModule,
    s: &mut AstroSlice,
    ctx: &mut StepCtx<'_>,
    id: u64,
    now: u64,
    dt_s: f64,
    t_mid_ns: i64,
) {
    let Some(c0) = s.craft.get(&id) else { return };
    if c0.status != FlightStatus::Propagating {
        return;
    }
    let dominant = c0.dominant;
    let engine = c0.engine.clone();
    let inline_engine = c0.inline_engine.clone();
    let throttle = c0.throttle;
    let guidance = c0.guidance.clone();
    let burn = s.burns.get(&id).cloned();

    // Frozen environment at the step midpoint (research R7).
    let env = {
        let mut states = BodyStates::new(&m.catalog, &s.motions, t_mid_ns);
        forces::freeze(&m.catalog, &mut states, &m.config, dominant)
    };
    // Resolve the engine: inline (designer-built) in preference to the catalogue.
    let def_owned = inline_engine.or_else(|| m.engines.get(&engine).cloned());
    let def = def_owned.as_ref();

    // Tier (research R2; state-driven only).
    let c = s.craft.get(&id).expect("craft");
    let tier = {
        if burn.is_some() {
            Tier::Burn
        } else if in_encounter_zone(m, dominant, c.state.r) {
            Tier::Encounter
        } else {
            Tier::Coast
        }
    };
    let sub_dt = match tier {
        Tier::Burn => m.config.burn_substep_s,
        Tier::Encounter => m.config.encounter_step_s as f64,
        Tier::Coast => dt_s, // single step over the cadence window
    };
    let n_sub = (dt_s / sub_dt).round().max(1.0) as u32;
    let sub_dt = dt_s / f64::from(n_sub); // exact integer subdivision

    let mut state = c.state;
    let mut mass = c.mass;
    let mut burn_state = burn;
    let mut exhausted = false;

    for _ in 0..n_sub {
        // Thrust for this substep.
        let mut thrust_dir = Vec3::ZERO;
        let mut thrust_n = 0.0;
        if let (Some(b), Some(def)) = (&burn_state, def) {
            let binding = EngineBinding {
                def,
                mass: &mut mass,
                throttle: 1.0,
                available_power_w: c0.available_power_w,
                drag_area_m2: c0.drag_area_m2,
                srp_area_m2: c0.srp_area_m2,
            };
            thrust_n = delivered_thrust(&binding, 1.0);
            thrust_dir = b.direction;
        } else if let (Some(arc), Some(def)) = (&guidance, def) {
            let dir = match arc.law {
                GuidanceLaw::Tangential { sign } => state.v.normalized() * f64::from(sign),
            };
            let binding = EngineBinding {
                def,
                mass: &mut mass,
                throttle,
                available_power_w: c0.available_power_w,
                drag_area_m2: c0.drag_area_m2,
                srp_area_m2: c0.srp_area_m2,
            };
            thrust_n = delivered_thrust(&binding, if throttle > 0.0 { throttle } else { 1.0 });
            thrust_dir = dir;
        }

        let total_mass = mass.total().max(1e-9);
        let mut dt_used = sub_dt;
        let mut accel_thrust = Vec3::ZERO;
        let mut a_used = 0.0;
        if thrust_n > 0.0 {
            let ve = def.expect("engine").exhaust_velocity();
            let mdot = thrust_n / ve;
            // Midpoint-mass acceleration keeps delivered Δv and the rocket
            // equation consistent to O(dt²) per substep (SC-005 0.1% honesty).
            let a_mid = |dt: f64| thrust_n / (total_mass - 0.5 * mdot * dt).max(1e-9);
            let mut a = a_mid(sub_dt);
            // Cut at burn completion (one fixed-point refinement of the cut time).
            if let Some(b) = &mut burn_state
                && a * sub_dt >= b.remaining_dv
            {
                let dt0 = (b.remaining_dv / a).max(0.0);
                a = a_mid(dt0);
                dt_used = (b.remaining_dv / a).max(0.0).min(sub_dt);
            }
            // Cut at propellant exhaustion.
            let prop_time = mass.propellant / mdot;
            if prop_time < dt_used {
                dt_used = prop_time;
                a = a_mid(dt_used);
                exhausted = true;
            }
            a_used = a;
            accel_thrust = thrust_dir * a;
        }

        let srp_aom = c0.srp_area_m2 / total_mass;
        let drag_aom = c0.drag_area_m2 / total_mass;
        let accel_fn = |r: Vec3, v: Vec3| forces::acceleration(&env, r, v, srp_aom, drag_aom);

        if dt_used > 0.0 {
            state = crate::integrator::rk4_step(&state, dt_used, accel_thrust, &accel_fn);
            if thrust_n > 0.0 {
                let ve = def.expect("engine").exhaust_velocity();
                let used = thrust_n * dt_used / ve;
                mass.propellant = (mass.propellant - used).max(0.0);
                if let Some(b) = &mut burn_state {
                    b.remaining_dv = (b.remaining_dv - a_used * dt_used).max(0.0);
                }
            }
        }
        // Remainder of the substep coasts (after cut/exhaustion).
        if dt_used < sub_dt {
            state = crate::integrator::rk4_step(&state, sub_dt - dt_used, Vec3::ZERO, &accel_fn);
        }
        if let Some(b) = &burn_state
            && b.remaining_dv <= 1e-9
        {
            burn_state = None;
        }
        if exhausted {
            break;
        }
    }

    // Write back craft dynamics.
    {
        let c = s.craft.get_mut(&id).expect("craft");
        c.state = state;
        c.mass = mass;
    }

    // Burn bookkeeping + exhaustion events.
    if exhausted {
        let shortfall = burn_state.as_ref().map(|b| b.remaining_dv).unwrap_or(0.0);
        s.burns.remove(&id);
        ctx.emit(
            "propellant-exhausted",
            BTreeMap::from([
                ("craft".to_string(), ViewValue::U64(id)),
                ("shortfall_dv".to_string(), ViewValue::F64(shortfall)),
            ]),
        );
        invalidate_plan(s, ctx, id, "propellant exhausted mid-burn");
    } else {
        match burn_state {
            Some(b) => {
                s.burns.insert(id, b);
            }
            None => {
                s.burns.remove(&id);
            }
        }
    }

    // Guidance arc end conditions.
    if let Some(c) = s.craft.get_mut(&id)
        && let Some(arc) = &c.guidance
    {
        let done = match arc.end {
            ArcEnd::TargetRadius(rt) => {
                let r = c.state.r.norm();
                let outward = matches!(arc.law, GuidanceLaw::Tangential { sign: 1 });
                if outward { r >= rt } else { r <= rt }
            }
            ArcEnd::AtTick(t) => now >= t,
        };
        if done || c.mass.propellant <= 0.0 {
            if c.mass.propellant <= 0.0 && !done {
                ctx.emit(
                    "propellant-exhausted",
                    BTreeMap::from([("craft".to_string(), ViewValue::U64(id))]),
                );
            }
            c.guidance = None;
        }
    }

    // SOI / surface / atmosphere bookkeeping at the boundary.
    surface_and_soi_checks(m, s, ctx, id, now);
}

fn integrate_diverted(
    m: &AstroModule,
    s: &mut AstroSlice,
    ctx: &mut StepCtx<'_>,
    id: BodyId,
    dt_s: f64,
    t_mid_ns: i64,
) {
    let Some(BodyMotion::Diverted { dominant, state }) = s.motions.get(&id).cloned() else {
        return;
    };
    let env = {
        let mut states = BodyStates::new(&m.catalog, &s.motions, t_mid_ns);
        forces::freeze(&m.catalog, &mut states, &m.config, dominant)
    };
    let accel_fn = |r: Vec3, v: Vec3| forces::acceleration(&env, r, v, 0.0, 0.0);
    let tier_dt = {
        if in_encounter_zone(m, dominant, state.r) {
            m.config.encounter_step_s as f64
        } else {
            dt_s
        }
    };
    let n = (dt_s / tier_dt).round().max(1.0) as u32;
    let sub = dt_s / f64::from(n);
    let mut st = state;
    for _ in 0..n {
        st = crate::integrator::rk4_step(&st, sub, Vec3::ZERO, &accel_fn);
    }
    // Dominant re-evaluation (a diverted body can change SOI like a craft).
    let t_now = t_mid_ns + ((dt_s / 2.0) * 1e9) as i64;
    let mut states = BodyStates::new(&m.catalog, &s.motions, t_now);
    let dom_abs = states.absolute(dominant);
    let abs_r = st.r + dom_abs.r;
    let mut new_dom = dominant_of(&m.catalog, &mut states, abs_r);
    if new_dom == id {
        new_dom = m
            .catalog
            .body(id)
            .and_then(|b| b.parent)
            .unwrap_or(m.catalog.root());
    }
    let final_state = if new_dom != dominant {
        let nd = states.absolute(new_dom);
        ctx.emit(
            "soi-crossing",
            BTreeMap::from([
                ("body".to_string(), ViewValue::U64(u64::from(id.0))),
                ("from".to_string(), ViewValue::U64(u64::from(dominant.0))),
                ("to".to_string(), ViewValue::U64(u64::from(new_dom.0))),
            ]),
        );
        StateVec {
            r: abs_r - nd.r,
            v: st.v + dom_abs.v - nd.v,
        }
    } else {
        st
    };
    s.motions.insert(
        id,
        BodyMotion::Diverted {
            dominant: new_dom,
            state: final_state,
        },
    );
}

/// Surface impact, atmosphere-entry handoff, planned-pass tracking, SOI rebase.
fn surface_and_soi_checks(
    m: &AstroModule,
    s: &mut AstroSlice,
    ctx: &mut StepCtx<'_>,
    id: u64,
    now: u64,
) {
    let t_ns = now as i64 * 1_000_000_000;
    let Some(c) = s.craft.get(&id) else { return };
    let dominant = c.dominant;
    let dom = m.catalog.body(dominant).expect("dominant");
    let rn = c.state.r.norm();

    // Impact (terminal here; vehicle slice consumes the handoff).
    if rn <= dom.radius {
        let craft = s.craft.get_mut(&id).expect("craft");
        craft.status = FlightStatus::Impacted {
            body: dominant,
            tick: now,
        };
        s.burns.remove(&id);
        ctx.emit(
            "impact",
            BTreeMap::from([
                ("craft".to_string(), ViewValue::U64(id)),
                ("body".to_string(), ViewValue::U64(u64::from(dominant.0))),
            ]),
        );
        invalidate_plan(s, ctx, id, "craft impacted");
        return;
    }

    // Atmosphere interface.
    if let Some(atmo) = &dom.atmosphere {
        let alt = rn - dom.radius;
        if alt < atmo.interface_alt_m {
            let in_planned_pass = s
                .aero_passes
                .get(&id)
                .map(|p| p.body == dominant && (p.start_tick..=p.end_tick).contains(&now))
                .unwrap_or(false);
            if in_planned_pass {
                let floor = m.config.aero_min_periapsis_alt_m;
                let pass = s.aero_passes.get_mut(&id).expect("pass");
                pass.min_altitude_m = pass.min_altitude_m.min(alt);
                if alt < floor && !pass.violated {
                    pass.violated = true;
                    ctx.emit(
                        "aero-violation",
                        BTreeMap::from([
                            ("craft".to_string(), ViewValue::U64(id)),
                            ("altitude_m".to_string(), ViewValue::F64(alt)),
                            ("floor_m".to_string(), ViewValue::F64(floor)),
                        ]),
                    );
                }
            } else {
                let craft = s.craft.get_mut(&id).expect("craft");
                craft.status = FlightStatus::EntryHandoff {
                    body: dominant,
                    tick: now,
                };
                s.burns.remove(&id);
                ctx.emit(
                    "atmosphere-entry",
                    BTreeMap::from([
                        ("craft".to_string(), ViewValue::U64(id)),
                        ("body".to_string(), ViewValue::U64(u64::from(dominant.0))),
                    ]),
                );
                invalidate_plan(s, ctx, id, "unplanned atmosphere entry");
                return;
            }
        }
    }
    // Expire finished aero-pass windows.
    if let Some(p) = s.aero_passes.get(&id)
        && now > p.end_tick
    {
        s.aero_passes.remove(&id);
    }

    // SOI handoff (continuity-exact rebase, research R6).
    let mut states = BodyStates::new(&m.catalog, &s.motions, t_ns);
    let dom_abs = states.absolute(dominant);
    let abs_r = c.state.r + dom_abs.r;
    let new_dom = dominant_of(&m.catalog, &mut states, abs_r);
    if new_dom != dominant {
        let nd = states.absolute(new_dom);
        let craft = s.craft.get_mut(&id).expect("craft");
        craft.state = StateVec {
            r: abs_r - nd.r,
            v: craft.state.v + dom_abs.v - nd.v,
        };
        craft.dominant = new_dom;
        ctx.emit(
            "soi-crossing",
            BTreeMap::from([
                ("craft".to_string(), ViewValue::U64(id)),
                ("from".to_string(), ViewValue::U64(u64::from(dominant.0))),
                ("to".to_string(), ViewValue::U64(u64::from(new_dom.0))),
            ]),
        );
    }
}

/// Station-keeping burn: hold the stored rotating-frame reference (research-noted
/// PD scheme): cancel velocity error and close position error over one cadence,
/// capped by the per-burn Δv budget. Consumes real propellant (rocket equation).
fn apply_station_keeping(m: &AstroModule, s: &mut AstroSlice, craft_id: u64, t_ns: i64) {
    let Some(sk) = s.station_keeping.get(&craft_id).cloned() else {
        return;
    };
    let Some(c) = s.craft.get(&craft_id) else {
        return;
    };
    if c.status != FlightStatus::Propagating {
        return;
    }
    let mut states = BodyStates::new(&m.catalog, &s.motions, t_ns);
    let (primary, secondary) = sk.rotating_pair;
    let dom_abs = states.absolute(c.dominant);
    let abs = StateVec {
        r: c.state.r + dom_abs.r,
        v: c.state.v + dom_abs.v,
    };
    // Where the reference sits now, in absolute terms.
    let p = states.absolute(primary);
    let sref = states.absolute(secondary);
    let x_hat = (sref.r - p.r).normalized();
    let z_hat = (sref.r - p.r).cross(sref.v - p.v).normalized();
    let y_hat = z_hat.cross(x_hat);
    let target_abs =
        p.r + x_hat * sk.reference_rot.x + y_hat * sk.reference_rot.y + z_hat * sk.reference_rot.z;
    // Angular velocity of the rotating frame carries the reference point.
    let omega = (sref.r - p.r).cross(sref.v - p.v) * (1.0 / (sref.r - p.r).norm2());
    let target_vel = p.v + omega.cross(target_abs - p.r);

    let pos_err = target_abs - abs.r;
    let vel_err = target_vel - abs.v;
    let cadence_s = sk.cadence_ticks as f64;
    let mut dv = vel_err + pos_err * (1.0 / cadence_s);
    if dv.norm() > sk.budget_dv {
        dv = dv.normalized() * sk.budget_dv;
    }
    // Apply impulsively; debit propellant by the rocket equation.
    let craft = s.craft.get_mut(&craft_id).expect("craft");
    if let Some(def) = craft
        .inline_engine
        .clone()
        .or_else(|| m.engines.get(&craft.engine).cloned())
        .as_ref()
    {
        let ve = def.exhaust_velocity();
        let m0 = craft.mass.total();
        let m1 = m0 * libm::exp(-dv.norm() / ve);
        let used = (m0 - m1).min(craft.mass.propellant);
        if used < craft.mass.propellant {
            craft.state.v = craft.state.v + dv;
            craft.mass.propellant -= used;
        }
    }
    let sk = s.station_keeping.get_mut(&craft_id).expect("sk");
    sk.next_tick += sk.cadence_ticks;
}

fn invalidate_plan(s: &mut AstroSlice, ctx: &mut StepCtx<'_>, craft_id: u64, reason: &str) {
    if let Some(plan) = s.plans.remove(&craft_id) {
        for n in &plan.nodes {
            if let Some(node) = s.nodes.get_mut(n)
                && !node.executed
            {
                node.committed = false;
            }
        }
        ctx.emit(
            "plan-invalidated",
            BTreeMap::from([
                ("craft".to_string(), ViewValue::U64(craft_id)),
                ("reason".to_string(), ViewValue::Str(reason.to_string())),
            ]),
        );
    }
}

/// Daily reconciliation (FR-ASTRO-402): conic-predict each planned craft to its
/// aim epoch and surface divergence past the threshold (diagnostic event).
fn reconcile_all(m: &AstroModule, s: &mut AstroSlice, ctx: &mut StepCtx<'_>, now: u64) {
    let t_ns = now as i64 * 1_000_000_000;
    let plans: Vec<(u64, AimPoint)> = s
        .plans
        .iter()
        .filter_map(|(c, p)| p.aim.map(|a| (*c, a)))
        .collect();
    for (craft_id, aim) in plans {
        let Some(c) = s.craft.get(&craft_id) else {
            continue;
        };
        if c.status != FlightStatus::Propagating || aim.epoch_tick <= now {
            continue;
        }
        let mut states = BodyStates::new(&m.catalog, &s.motions, t_ns);
        let dom_abs = states.absolute(c.dominant);
        let abs = StateVec {
            r: c.state.r + dom_abs.r,
            v: c.state.v + dom_abs.v,
        };
        let root_mu = m.catalog.body(m.catalog.root()).expect("root").mu;
        let coast = (aim.epoch_tick - now) as f64;
        let Some(pred) = crate::math::universal_propagate(root_mu, &abs, coast) else {
            continue;
        };
        let aim_t = aim.epoch_tick as i64 * 1_000_000_000;
        let mut states_aim = BodyStates::new(&m.catalog, &s.motions, aim_t);
        let body_pos = states_aim.absolute(aim.body).r;
        let miss = (pred.r - body_pos).norm();
        let divergence = (miss - aim.target_radius_m).max(0.0);
        if divergence > m.config.divergence_threshold_m {
            ctx.emit(
                "kernel-diagnostic",
                BTreeMap::from([
                    ("kind".to_string(), ViewValue::Str("divergence".into())),
                    ("craft".to_string(), ViewValue::U64(craft_id)),
                    ("miss_m".to_string(), ViewValue::F64(miss)),
                    (
                        "threshold_m".to_string(),
                        ViewValue::F64(m.config.divergence_threshold_m),
                    ),
                ]),
            );
        }
    }
}

// ----------------------------------------------------------------- commands

#[allow(clippy::too_many_lines)]
fn apply_command(
    m: &AstroModule,
    s: &mut AstroSlice,
    ctx: &mut StepCtx<'_>,
    cmd: AstroCommand,
) -> CommandOutcome {
    use AstroCommand::*;
    let now = ctx.tick().0;
    let reject = |r: String| CommandOutcome::Rejected(r);
    match cmd {
        SpawnCraft {
            name_id,
            dominant,
            state,
            dry_mass,
            propellant,
            engine,
            inline_engine,
            available_power_w,
            drag_area_m2,
            srp_area_m2,
        } => {
            if state.r.non_finite() || state.v.non_finite() || !dry_mass.is_finite() {
                return reject("non-finite spawn state".into());
            }
            if dry_mass <= 0.0 || propellant < 0.0 {
                return reject("masses must be positive".into());
            }
            if m.catalog.body(BodyId(dominant)).is_none() {
                return reject(format!("unknown dominant body {dominant}"));
            }
            // Designer-built engines are carried inline (FA-04); otherwise the
            // engine id must resolve against the fixture catalogue.
            match &inline_engine {
                Some(ie) if ie.isp_s <= 0.0 || ie.max_thrust_n <= 0.0 => {
                    return reject("inline engine: Isp and thrust must be positive".into());
                }
                Some(_) => {}
                None => {
                    if m.engines.get(&engine).is_none() {
                        return reject(format!("unknown engine '{engine}'"));
                    }
                }
            }
            let id = s.next_craft;
            s.next_craft += 1;
            s.craft.insert(
                id,
                Craft {
                    id: CraftId(id),
                    name_id,
                    dominant: BodyId(dominant),
                    state,
                    mass: MassState {
                        dry: dry_mass,
                        propellant,
                    },
                    engine,
                    inline_engine,
                    throttle: 1.0,
                    available_power_w,
                    drag_area_m2,
                    srp_area_m2,
                    guidance: None,
                    status: FlightStatus::Propagating,
                },
            );
            CommandOutcome::Applied
        }
        DespawnCraft { craft } => {
            if s.craft.remove(&craft).is_none() {
                return reject(format!("unknown craft {craft}"));
            }
            s.burns.remove(&craft);
            s.plans.remove(&craft);
            s.station_keeping.remove(&craft);
            s.aero_passes.remove(&craft);
            CommandOutcome::Applied
        }
        CreateNode {
            craft,
            epoch_tick,
            dv_prn,
        } => {
            if dv_prn.non_finite() {
                return reject("non-finite dv".into());
            }
            if !s.craft.contains_key(&craft) {
                return reject(format!("unknown craft {craft}"));
            }
            if epoch_tick <= now {
                return reject(format!("node epoch {epoch_tick} is not in the future"));
            }
            let id = s.next_node;
            s.next_node += 1;
            s.nodes.insert(
                id,
                ManeuverNode {
                    id,
                    craft: CraftId(craft),
                    epoch_tick,
                    dv_prn,
                    committed: false,
                    executed: false,
                },
            );
            CommandOutcome::Applied
        }
        EditNode {
            node,
            epoch_tick,
            dv_prn,
        } => {
            if dv_prn.non_finite() {
                return reject("non-finite dv".into());
            }
            let Some(n) = s.nodes.get_mut(&node) else {
                return reject(format!("unknown node {node}"));
            };
            if n.committed {
                return reject("cannot edit a committed node".into());
            }
            n.epoch_tick = epoch_tick;
            n.dv_prn = dv_prn;
            CommandOutcome::Applied
        }
        DeleteNode { node } => {
            let Some(n) = s.nodes.get(&node) else {
                return reject(format!("unknown node {node}"));
            };
            if n.committed && !n.executed {
                return reject("cannot delete a committed node (plans first)".into());
            }
            s.nodes.remove(&node);
            CommandOutcome::Applied
        }
        CommitPlan { craft, nodes, aim } => {
            if !s.craft.contains_key(&craft) {
                return reject(format!("unknown craft {craft}"));
            }
            if nodes.is_empty() {
                return reject("a plan needs at least one node".into());
            }
            let mut last_epoch = now;
            for nid in &nodes {
                let Some(n) = s.nodes.get(nid) else {
                    return reject(format!("unknown node {nid}"));
                };
                if n.craft.0 != craft {
                    return reject(format!("node {nid} belongs to another craft"));
                }
                if n.epoch_tick <= last_epoch {
                    return reject("plan nodes must be strictly time-ordered".into());
                }
                last_epoch = n.epoch_tick;
            }
            for nid in &nodes {
                let n = s.nodes.get_mut(nid).expect("checked");
                n.committed = true;
                // Kernel pause at the node epoch (FR-ASTRO-301).
                ctx.schedule(
                    Tick(n.epoch_tick),
                    "maneuver-node",
                    BTreeMap::from([
                        ("node".to_string(), ViewValue::U64(*nid)),
                        ("craft".to_string(), ViewValue::U64(craft)),
                    ]),
                );
            }
            s.plans.insert(craft, Plan { nodes, aim });
            CommandOutcome::Applied
        }
        SetThrottle { craft, throttle } => {
            if !(0.0..=1.0).contains(&throttle) {
                return reject("throttle must be in [0,1]".into());
            }
            let Some(c) = s.craft.get_mut(&craft) else {
                return reject(format!("unknown craft {craft}"));
            };
            c.throttle = throttle;
            CommandOutcome::Applied
        }
        SetGuidanceArc { craft, law, end } => {
            let Some(c) = s.craft.get_mut(&craft) else {
                return reject(format!("unknown craft {craft}"));
            };
            if c.status != FlightStatus::Propagating {
                return reject("craft is not propagating".into());
            }
            c.guidance = Some(crate::craft::GuidanceArc { law, end });
            CommandOutcome::Applied
        }
        ClearGuidanceArc { craft } => {
            let Some(c) = s.craft.get_mut(&craft) else {
                return reject(format!("unknown craft {craft}"));
            };
            c.guidance = None;
            CommandOutcome::Applied
        }
        ScheduleStationKeeping {
            craft,
            cadence_ticks,
            budget_dv,
            rotating_pair,
        } => {
            if cadence_ticks == 0 || budget_dv <= 0.0 {
                return reject("cadence and budget must be positive".into());
            }
            let Some(c) = s.craft.get(&craft) else {
                return reject(format!("unknown craft {craft}"));
            };
            let (p, sec) = (BodyId(rotating_pair.0), BodyId(rotating_pair.1));
            if m.catalog.body(p).is_none() || m.catalog.body(sec).is_none() {
                return reject("unknown rotating pair".into());
            }
            // Reference = current rotating-frame position.
            let t_ns = now as i64 * 1_000_000_000;
            let mut states = BodyStates::new(&m.catalog, &s.motions, t_ns);
            let dom_abs = states.absolute(c.dominant);
            let abs_r = c.state.r + dom_abs.r;
            let reference_rot = crate::frames::to_rotating(&mut states, p, sec, abs_r);
            s.station_keeping.insert(
                craft,
                StationKeeping {
                    cadence_ticks,
                    budget_dv,
                    rotating_pair: (p, sec),
                    reference_rot,
                    next_tick: now + cadence_ticks,
                },
            );
            CommandOutcome::Applied
        }
        CancelStationKeeping { craft } => {
            if s.station_keeping.remove(&craft).is_none() {
                return reject(format!("no station-keeping for craft {craft}"));
            }
            CommandOutcome::Applied
        }
        PlanAeroPass {
            craft,
            body,
            start_tick,
            end_tick,
        } => {
            if !s.craft.contains_key(&craft) {
                return reject(format!("unknown craft {craft}"));
            }
            let b = BodyId(body);
            let Some(def) = m.catalog.body(b) else {
                return reject(format!("unknown body {body}"));
            };
            if def.atmosphere.is_none() {
                return reject(format!("body '{}' has no atmosphere", def.name_id));
            }
            if end_tick <= start_tick {
                return reject("aero-pass window must be non-empty".into());
            }
            s.aero_passes.insert(
                craft,
                AeroPassPlan {
                    body: b,
                    start_tick,
                    end_tick,
                    min_altitude_m: f64::INFINITY,
                    violated: false,
                },
            );
            CommandOutcome::Applied
        }
        DivertBody { body, impulse } => {
            let id = BodyId(body);
            let t_ns = now as i64 * 1_000_000_000;
            match divert::divert(
                &m.catalog,
                &s.motions,
                m.config.diversion_budget,
                id,
                impulse,
                t_ns,
            ) {
                Ok(motion) => {
                    s.motions.insert(id, motion);
                    CommandOutcome::Applied
                }
                Err(e) => reject(e),
            }
        }
        ReRailBody { body } => {
            let id = BodyId(body);
            let t_ns = now as i64 * 1_000_000_000;
            match divert::re_rail(&m.catalog, &s.motions, id, t_ns) {
                Ok(motion) => {
                    s.motions.insert(id, motion);
                    CommandOutcome::Applied
                }
                Err(e) => reject(e),
            }
        }
        SetResearchGate { gate, open } => {
            s.gates.insert(gate, open);
            CommandOutcome::Applied
        }
    }
}
