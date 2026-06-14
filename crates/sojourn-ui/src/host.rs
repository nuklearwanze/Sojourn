//! The in-process UI host (R3, `contracts/ui-host.md`). `UiHost` owns a `SimCore`
//! with the full module set, drives time-warp stepping, exposes the event/interrupt
//! feed and typed snapshot pulls, and submits typed journalled commands. It holds
//! **no game logic** and **never blocks or alters** the core's deterministic stepping;
//! the same `SimCore` runs headless in the harness.

use crate::command::CommitOutcome;
use crate::modules::build_modules;
use sojourn_astro::bodies::rails::BodyStates;
use sojourn_astro::math::elements_to_state;
use sojourn_astro::{AstroModule, AstroSnapshot, BodyId, Elements};
use sojourn_base::BaseModule;
use sojourn_base::catalogue::ModuleParams as BaseModuleParams;
use sojourn_core::{
    Command, CommandOutcome, CoreError, DataSet, EventFilter, RunConfig, RunMode, SimCore,
    StepRequest,
};
use sojourn_economy::commodity::{CommodityKind, Unit};
use sojourn_economy::funding::FundingProfile;
use sojourn_economy::ids::FactionId as EFactionId;
use sojourn_economy::ledger::Currency;
use sojourn_economy::query::EconomySnapshot;
use sojourn_economy::{EconomyInputs, EconomyModule};
use sojourn_polity::PolityModule;
use sojourn_polity::ids::FactionId as PFactionId;
use sojourn_polity::query::WorldSnapshot;
use sojourn_research::{FactionId, ResearchModule, ResearchSnapshot, TechId};
use sojourn_vehicle::VehicleModule;
use sojourn_vehicle::catalogue::{Component, ComponentParams};
use sojourn_vehicle::design::{MissionReqs, Stage, VehicleDesign};
use sojourn_vehicle::ids::{ComponentId, DesignId, FactionId as VFactionId};
use sojourn_vehicle::query::{ComponentMaturity, DesignSnapshot};
use std::collections::BTreeMap;
use std::path::Path;

/// One astronomical unit in metres — the reference solar distance the Vehicle
/// Designer derives power/EDL at (FR-VD-201).
const AU_M: f64 = 1.495_978_707e11;

/// A body for the System Map, read from the astro/world surfaces (heliocentric).
#[derive(Debug, Clone, PartialEq)]
pub struct MapBody {
    /// Catalogue id.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Heliocentric ecliptic position (m).
    pub pos: (f64, f64),
    /// Mean radius (m).
    pub radius: f64,
    /// The body's **real orbital path** (absolute heliocentric ecliptic m), sampled
    /// over one full period from its elements — empty for the root, hyperbolic
    /// bodies, or insignificant small bodies (kept as dots only).
    pub orbit: Vec<(f64, f64)>,
}

/// A body's catalogue facts gathered before the mutable rail evaluator is built:
/// `(id, display name, mean radius, parent, elements)`.
type BodyFacts = (BodyId, String, f64, Option<BodyId>, Option<Elements>);

/// Sample a closed orbit ellipse (absolute m, ecliptic projection) over one period.
fn sample_orbit(mu: f64, el: &Elements, parent_abs: (f64, f64)) -> Vec<(f64, f64)> {
    if mu <= 0.0 || el.sma <= 0.0 || el.ecc >= 1.0 {
        return Vec::new();
    }
    const N: usize = 96;
    let period_s = 2.0 * std::f64::consts::PI * libm::sqrt(el.sma * el.sma * el.sma / mu);
    let step_ns = period_s * 1.0e9 / N as f64;
    (0..=N)
        .map(|k| {
            let t_ns = el.epoch_ns + (k as f64 * step_ns) as i64;
            let s = elements_to_state(mu, el, t_ns);
            (parent_abs.0 + s.r.x, parent_abs.1 + s.r.y)
        })
        .collect()
}

/// A knowledge domain for the R&D screen (FR-UI-301): the player faction's
/// Understanding Levels read straight from the FA-05 research surface, plus the
/// domain's static shape (classification, diminishing-returns knee). The UI derives
/// nothing — every figure here is a research-slice query result.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainView {
    /// Domain key (`"A1"`).
    pub id: String,
    /// Display name (e.g. "Materials").
    pub name: String,
    /// The faction's private (ground) Understanding Level, 0–100.
    pub private_ul: f64,
    /// The shared world tide (UL the field has reached publicly), 0–100.
    pub world_ul: f64,
    /// Effective UL gates actually use (tacit-knowledge adjusted), 0–100.
    pub effective_ul: f64,
    /// Basic vs applied science (insight-pressure weighting).
    pub basic_science: bool,
    /// Whether missions inject UL here (the A8–A16 mission-fed fields).
    pub mission_injectable: bool,
    /// UL above which growth cost rises sharply (the diminishing-returns knee).
    pub dr_knee: f64,
}

/// Display-name a `dom.*` identifier: drop the `dom.` namespace, title-case the rest.
fn domain_name(name_id: &str) -> String {
    let bare = name_id.strip_prefix("dom.").unwrap_or(name_id);
    display_name(bare)
}

/// One catalogued component for the Vehicle Designer's parts palette (FR-VD-101):
/// its class, the FA-05 technology that researches it, dry mass, a one-line spec and
/// the technology's current maturity (the C1-decoupled FA-05 read — all unflyable on
/// a fresh run until the tech is researched).
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleComponent {
    /// Component data key.
    pub id: String,
    /// Component class (e.g. "Engine", "Tank").
    pub class: String,
    /// FA-05 technology key (e.g. "B1.4").
    pub tech: String,
    /// Structural dry mass (kg).
    pub dry_mass: f64,
    /// A one-line class-specific spec (e.g. "Isp 450 s · 110 kN").
    pub detail: String,
    /// Current TRL (from FA-05; 0 = unresearched).
    pub trl: u8,
    /// Flyable (TRL ≥ 6)?
    pub flyable: bool,
}

/// A realism red-flag flattened for display (FR-VD-601): the violated constraint, the
/// offending value, and whether it is hard (impossible) or soft (marginal-but-built).
#[derive(Debug, Clone, PartialEq)]
pub struct RedFlagView {
    /// The violated constraint.
    pub constraint: String,
    /// The offending value.
    pub value: f64,
    /// Hard (physically impossible) vs soft (risky-but-buildable).
    pub hard: bool,
}

/// The Vehicle Designer's derived report for a reference design (FR-VD-201/501/601).
/// **Every number is the vehicle slice's derivation** at 1 AU — the UI computes none
/// of it. On a fresh run the reference airframe's mass/Δv/power are real, while its
/// reliability is low and its red-flags fire because no propulsion tech is researched
/// yet (the C1 FA-05 maturity read), which is exactly what the screen should teach.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleReport {
    /// Display name of the reference design.
    pub name: String,
    /// Class template.
    pub class: String,
    /// Dry mass (kg).
    pub dry_mass: f64,
    /// Wet mass (kg).
    pub wet_mass: f64,
    /// Propellant (kg).
    pub propellant: f64,
    /// Total Δv (m/s).
    pub total_dv: f64,
    /// Per-stage Δv (m/s).
    pub stage_dvs: Vec<f64>,
    /// First-stage thrust (N).
    pub max_thrust: f64,
    /// Power generation at 1 AU (W).
    pub power_gen: f64,
    /// Power demand (W).
    pub power_demand: f64,
    /// Power margin (W).
    pub power_margin: f64,
    /// Thermal margin (W).
    pub thermal_margin: f64,
    /// Composed reliability ∈ [0,1].
    pub reliability: f64,
    /// Unit cost estimate (abstract units).
    pub unit_cost: f64,
    /// Build time (days).
    pub build_days: f64,
    /// Realism red-flags.
    pub red_flags: Vec<RedFlagView>,
    /// The component palette (every catalogued part + maturity).
    pub components: Vec<VehicleComponent>,
}

/// The class label for a component's parameters.
fn component_class(p: &ComponentParams) -> &'static str {
    match p {
        ComponentParams::Structure { .. } => "Structure",
        ComponentParams::Tank { .. } => "Tank",
        ComponentParams::Power { .. } => "Power",
        ComponentParams::Thermal { .. } => "Thermal",
        ComponentParams::Avionics { .. } => "Avionics",
        ComponentParams::Comms { .. } => "Comms",
        ComponentParams::LifeSupport { .. } => "Life support",
        ComponentParams::Payload { .. } => "Payload",
        ComponentParams::EdlKit { .. } => "EDL kit",
        ComponentParams::Landing => "Landing",
        ComponentParams::Rcs => "RCS",
        ComponentParams::Docking => "Docking",
        ComponentParams::Engine { .. } => "Engine",
    }
}

/// A one-line spec for a component (the headline derived/raw figures).
fn component_detail(c: &Component) -> String {
    match &c.params {
        ComponentParams::Structure { thrust_limit_n } => {
            format!("rated {:.0} kN", thrust_limit_n / 1.0e3)
        }
        ComponentParams::Tank { capacity_kg, .. } => format!("cap {:.0} t", capacity_kg / 1.0e3),
        ComponentParams::Power { gen_w, solar } => format!(
            "{:.0} kW {}",
            gen_w / 1.0e3,
            if *solar { "solar" } else { "constant" }
        ),
        ComponentParams::Thermal { reject_w_per_kg } => format!("{reject_w_per_kg:.0} W/kg reject"),
        ComponentParams::Avionics { power_demand_w } => format!("{power_demand_w:.0} W"),
        ComponentParams::Comms { power_demand_w } => format!("{power_demand_w:.0} W"),
        ComponentParams::LifeSupport {
            crew,
            closure_fraction,
            ..
        } => {
            format!("{crew} crew · {:.0}% closed", closure_fraction * 100.0)
        }
        ComponentParams::Payload { power_demand_w, .. } => format!("{power_demand_w:.0} W"),
        ComponentParams::EdlKit { area_m2, .. } => format!("{area_m2:.0} m² shield"),
        ComponentParams::Landing => "landing gear".into(),
        ComponentParams::Rcs => "reaction control".into(),
        ComponentParams::Docking => "docking".into(),
        ComponentParams::Engine {
            isp_s,
            max_thrust_n,
            family,
            ..
        } => format!(
            "{:?} · Isp {:.0} s · {}",
            family,
            isp_s,
            if *max_thrust_n >= 1.0e3 {
                format!("{:.0} kN", max_thrust_n / 1.0e3)
            } else {
                format!("{max_thrust_n:.2} N")
            }
        ),
    }
}

/// A representative reference design (a robotic deep-space science probe) the Vehicle
/// Designer derives so the screen shows real figures before the player has authored
/// any design. Built from real catalogue parts; not a player design (id stays 0).
fn reference_design() -> VehicleDesign {
    let cid = |s: &str| ComponentId(s.to_string());
    let comps = [
        "struct-allium",
        "tank-cryo",
        "eng-hydrolox",
        "pwr-pv",
        "avionics",
        "comms-ka",
        "payload-sci",
    ];
    VehicleDesign {
        id: DesignId(0),
        faction: VFactionId(0),
        class: "probe".to_string(),
        stages: vec![Stage {
            components: comps.iter().map(|s| cid(s)).collect(),
            propellant_kg: 20_000.0,
            engine: Some(cid("eng-hydrolox")),
        }],
        redundancy: Vec::new(),
        mission: MissionReqs {
            dv: Some(6_000.0),
            mission_days: Some(400.0),
            ..Default::default()
        },
        parent: None,
        production_count: 1,
    }
}

/// Title-case an identifier-style `name_id` into a display name (FR-UI: display text
/// is the UI's concern; the catalogue stores ids).
fn display_name(name_id: &str) -> String {
    let mut out = String::new();
    for (i, part) in name_id.split(['-', '_', ' ']).enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// One conserved-currency balance for the Economy ledger (FR-EC-101): the six-currency
/// model the whole economy is denominated in (no fiat printing — everything conserves).
#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyBalance {
    /// Display name of the currency.
    pub name: String,
    /// Current balance for the player faction.
    pub amount: f64,
    /// Is this the money currency (formatted as currency, not a bare scalar)?
    pub is_funds: bool,
}

/// A launch-market price point (FR-EC-601): $/kg to an orbit class at the current
/// world-capacity index (cheaper as reusable capacity grows).
#[derive(Debug, Clone, PartialEq)]
pub struct LaunchPrice {
    /// Orbit class (e.g. "leo", "gto").
    pub orbit_class: String,
    /// Current price ($/kg).
    pub price_per_kg: f64,
}

/// One tradable commodity for the taxonomy table (FR-EC-107).
#[derive(Debug, Clone, PartialEq)]
pub struct CommodityRow {
    /// Stable commodity id.
    pub id: String,
    /// Kind label (Raw/Processed/Manufactured/Consumable/Strategic/Service).
    pub kind: String,
    /// Unit of measure (kg/each/service).
    pub unit: String,
}

/// The Economy & Contracts overview (S6, FR-EC-101/301/601/107): the player faction's
/// conserved-currency ledger, its funding profile, the live launch market and the
/// commodity taxonomy — all read from the economy surface; the UI computes nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct EconomyView {
    /// The six conserved-currency balances.
    pub balances: Vec<CurrencyBalance>,
    /// Funding archetype (Agency vs Private).
    pub funding_kind: String,
    /// Funding headline figures.
    pub funding_headline: String,
    /// Launch-market prices by orbit class (ascending).
    pub launch_prices: Vec<LaunchPrice>,
    /// The tradable-commodity taxonomy.
    pub commodities: Vec<CommodityRow>,
}

/// Display name for a conserved currency.
fn currency_label(c: Currency) -> &'static str {
    match c {
        Currency::Funds => "Funds",
        Currency::DeltaV => "Δv budget",
        Currency::MassToOrbit => "Mass-to-orbit",
        Currency::CrewTime => "Crew-time",
        Currency::OpsCapacity => "Ops capacity",
        Currency::PoliticalCapital => "Political capital",
    }
}

/// A label for a commodity's kind (the taxonomy class + any cross-reference).
fn commodity_kind_label(k: &CommodityKind) -> String {
    match k {
        CommodityKind::Raw { resource_ref } => format!("Raw · {resource_ref}"),
        CommodityKind::Processed { from } => format!("Processed ← {}", from.0),
        CommodityKind::Manufactured => "Manufactured".to_string(),
        CommodityKind::Consumable => "Consumable".to_string(),
        CommodityKind::Strategic { cap_ref } => format!("Strategic · {cap_ref}"),
        CommodityKind::Service => "Service".to_string(),
    }
}

/// A label for a commodity's unit.
fn unit_label(u: Unit) -> &'static str {
    match u {
        Unit::Kg => "kg",
        Unit::Each => "each",
        Unit::Service => "service",
    }
}

/// One historic-first milestone for the World/Politics race board (FR-PEA-101): its
/// era, description, significance weight and world-first claimant (if any).
#[derive(Debug, Clone, PartialEq)]
pub struct MilestoneRow {
    /// Milestone id.
    pub id: u32,
    /// Era (foothold/cislunar/frontier/endgame).
    pub era: String,
    /// Human description of the first.
    pub description: String,
    /// Significance weight (prestige on claim).
    pub weight: f64,
    /// World-first claimant faction id, if already claimed.
    pub claimed_by: Option<u32>,
}

/// One faction's political state (FR-PEA-201): mood, accrued prestige, and the
/// mood-derived budget/approval modifiers. Empty until the world is initialised.
#[derive(Debug, Clone, PartialEq)]
pub struct FactionPolitics {
    /// Faction id.
    pub id: u32,
    /// Current mood (bounded; baseline 0).
    pub mood: f64,
    /// Accrued prestige.
    pub prestige: f64,
    /// Mood-derived appropriation factor (agencies).
    pub appropriation_factor: f64,
    /// Mood-derived major-program approval latency (days).
    pub approval_latency_days: f64,
}

/// The World & Politics overview (S9, FR-PEA-101/201): the historic-firsts race board,
/// the global science tide, and per-faction political state — all read from the FA-09
/// polity surface. Factions appear once the world is initialised; the milestone
/// catalogue (and its unclaimed state) is live from the first frame.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldView {
    /// The milestone race board (full catalogue + claim state).
    pub milestones: Vec<MilestoneRow>,
    /// The global science-tide level.
    pub science_tide: f64,
    /// Per-faction political state (empty before world init).
    pub factions: Vec<FactionPolitics>,
}

/// A heliocentric Hohmann transfer option from the player's home world for the
/// Trajectory planner (S2): the destination, the two-burn Δv and the half-ellipse
/// time of flight, computed from the FA-02 catalogue. Real two-body physics — the
/// first cut of the porkchop planner before per-window optimisation.
#[derive(Debug, Clone, PartialEq)]
pub struct TransferOption {
    /// Destination body display name.
    pub to: String,
    /// Total two-burn Δv (m/s).
    pub dv_mps: f64,
    /// Hohmann time of flight (days).
    pub tof_days: f64,
    /// Destination heliocentric radius (AU).
    pub r_to_au: f64,
}

/// One craft for the Operations/Fleet screen (S5), read from the FA-02 craft store.
#[derive(Debug, Clone, PartialEq)]
pub struct CraftRow {
    /// Craft id.
    pub id: u64,
    /// Display name.
    pub name: String,
    /// Dominant body (SOI owner) name.
    pub dominant: String,
    /// Flight status.
    pub status: String,
}

/// One personnel role tally for the Personnel screen (S8).
#[derive(Debug, Clone, PartialEq)]
pub struct PersonnelRow {
    /// Role name.
    pub role: String,
    /// Head-count.
    pub count: u64,
}

/// One catalogued base module type for the Bases screen (S7, FR-BC-101).
#[derive(Debug, Clone, PartialEq)]
pub struct BaseModuleRow {
    /// Module type id.
    pub id: String,
    /// Module class (Habitat/Power/Eclss/…).
    pub class: String,
    /// FA-05 technology key.
    pub tech: String,
    /// Dry mass (kg).
    pub dry_mass: f64,
    /// Power demand (W).
    pub power_demand: f64,
}

/// One base-class template for the Bases screen (S7).
#[derive(Debug, Clone, PartialEq)]
pub struct BaseClassRow {
    /// Class id.
    pub id: String,
    /// Orbital (vs surface)?
    pub orbital: bool,
    /// Crewed?
    pub crewed: bool,
}

/// The Bases & Construction catalogue (S7): the module + class catalogue (live from
/// the first frame) plus the count of bases founded so far.
#[derive(Debug, Clone, PartialEq)]
pub struct BaseCatalogueView {
    /// The buildable module types.
    pub modules: Vec<BaseModuleRow>,
    /// The base-class templates.
    pub classes: Vec<BaseClassRow>,
    /// Bases founded so far (0 on a fresh programme).
    pub founded: usize,
}

/// One source-cited Sojournal entry (S11): the encyclopedia is assembled from the
/// provenance carried by every datum the game loads — every figure traces to a source.
#[derive(Debug, Clone, PartialEq)]
pub struct SojournalEntry {
    /// Category (Knowledge domain / Vehicle component / …).
    pub category: String,
    /// Entry title.
    pub title: String,
    /// The cited source.
    pub source: String,
}

/// The class label for a base module's parameters.
fn base_module_class(p: &BaseModuleParams) -> &'static str {
    match p {
        BaseModuleParams::Habitat { .. } => "Habitat",
        BaseModuleParams::Power { .. } => "Power",
        BaseModuleParams::Eclss { .. } => "ECLSS",
        BaseModuleParams::Greenhouse { .. } => "Greenhouse",
        BaseModuleParams::IsruHost { .. } => "ISRU host",
        BaseModuleParams::Science { .. } => "Science",
        BaseModuleParams::Storage { .. } => "Storage",
        BaseModuleParams::Manufacturing { .. } => "Manufacturing",
        BaseModuleParams::Shielding { .. } => "Shielding",
    }
}

/// A compact, player-facing run status (the clock every snapshot matches).
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    /// Current tick.
    pub tick: u64,
    /// Civil date (year, month, day).
    pub date: (i32, u8, u8),
    /// Total events emitted so far.
    pub total_events: u64,
    /// Pending (un-acknowledged) interrupts.
    pub pending_interrupts: u64,
}

/// A compact view of one event for the feed (FR-UI-1001).
#[derive(Debug, Clone, PartialEq)]
pub struct EventView {
    /// Sequential id (for acknowledge / linking).
    pub id: u64,
    /// Tick.
    pub tick: u64,
    /// Event class.
    pub class: String,
}

/// The in-process host: a `SimCore` plus retained module handles (for the typed
/// snapshot extractors) and the read/command façade.
pub struct UiHost {
    core: SimCore,
    astro: AstroModule,
    research: ResearchModule,
    vehicle: VehicleModule,
    economy: EconomyModule,
    polity: PolityModule,
    base: BaseModule,
}

impl UiHost {
    /// Start a fresh game: build the full module set under `data_root` and create the
    /// core. `data_root` is the repo root (holds `data/kernel`, `data/world`, …). A
    /// second `AstroModule` handle is retained for snapshot extraction (the catalogue
    /// is immutable data; the live state comes from the core's slice).
    pub fn new_game(data_root: &Path, seed: u64) -> Result<UiHost, String> {
        let data = DataSet::load_dir(&data_root.join("data/kernel"))
            .map_err(|e| format!("kernel data: {e}"))?;
        let cfg = RunConfig {
            master_seed: seed,
            horizon_years: 100,
            run_mode: RunMode::SaveAnywhere,
            difficulty_inputs: BTreeMap::new(),
        };
        let modules = build_modules(data_root)?;
        let core = SimCore::create(cfg, data, modules).map_err(|e| format!("core: {e}"))?;
        let astro = AstroModule::load(&data_root.join("data/world"))
            .map_err(|e| format!("astro handle: {e}"))?;
        let research = ResearchModule::load(&data_root.join("data"))
            .map_err(|e| format!("research handle: {e}"))?;
        let vehicle = VehicleModule::load(&data_root.join("data/vehicle"))
            .map_err(|e| format!("vehicle handle: {e}"))?;
        let economy = EconomyModule::load(&data_root.join("data/econ"))
            .map_err(|e| format!("economy handle: {e}"))?;
        let polity = PolityModule::load(&data_root.join("data/polity"))
            .map_err(|e| format!("polity handle: {e}"))?;
        let base = BaseModule::load(&data_root.join("data/base"))
            .map_err(|e| format!("base handle: {e}"))?;
        Ok(UiHost {
            core,
            astro,
            research,
            vehicle,
            economy,
            polity,
            base,
        })
    }

    /// Borrow the core for a typed in-process snapshot pull (e.g. `WorldSnapshot::
    /// from_core`, `CrewSnapshot::from_core`, or a `with_slice` read). The UI consumes
    /// the slices' read-only surfaces directly (FR-UI-1502).
    pub fn core(&self) -> &SimCore {
        &self.core
    }

    /// The System Map's bodies — real heliocentric positions at the current tick,
    /// propagated from the FA-02 railed ephemerides; names from the FA-03 catalogue.
    /// The UI computes **no** physics here: positions come from the astro surface; the
    /// renderer projects + LOD-culls them (FR-UI-102/1502).
    pub fn map_bodies(&self) -> Vec<MapBody> {
        let snap = match AstroSnapshot::from_core(&self.core, &self.astro) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        // (id, name, radius, parent, elements) — gathered first so the catalogue
        // borrow is released before the mutable `BodyStates` evaluator is built.
        let defs: Vec<BodyFacts> = snap
            .catalog
            .bodies()
            .map(|b| {
                (
                    b.id,
                    display_name(&b.name_id),
                    b.radius,
                    b.parent,
                    b.elements,
                )
            })
            .collect();
        let mut states = BodyStates::new(&snap.catalog, &snap.motions, snap.t_ns);
        let mut out = Vec::with_capacity(defs.len());
        for (id, name, radius, parent, elements) in defs {
            let r = states.absolute(id).r;
            // The real orbital path for every body with closed (elliptical) elements;
            // the renderer styles planet vs minor-body lines differently.
            let orbit = match (parent, &elements) {
                (Some(parent), Some(el)) => {
                    let mu = snap.catalog.body(parent).map(|b| b.mu).unwrap_or(0.0);
                    let parent_abs = states.absolute(parent).r;
                    sample_orbit(mu, el, (parent_abs.x, parent_abs.y))
                }
                _ => Vec::new(),
            };
            out.push(MapBody {
                id: id.0,
                name,
                pos: (r.x, r.y),
                radius,
                orbit,
            });
        }
        out
    }

    /// The R&D screen's knowledge domains — the player faction's Understanding
    /// Levels read from the FA-05 research surface, paired with each domain's static
    /// shape. The UI computes **no** science here: ULs come from the research slice's
    /// query surface; the renderer only draws the bars (FR-UI-301/1502). The player
    /// is faction 0 by convention; on a fresh run every UL is 0 (nothing researched).
    pub fn research_domains(&self) -> Vec<DomainView> {
        let snap = match ResearchSnapshot::from_core(&self.core, &self.research) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let f = FactionId(0);
        self.research
            .data
            .domains
            .values()
            .map(|d| {
                let u = snap.understanding(f, &d.id);
                DomainView {
                    id: d.id.0.clone(),
                    name: domain_name(&d.name_id),
                    private_ul: u.map(|u| u.private_ul).unwrap_or(0.0),
                    world_ul: u.map(|u| u.world_ul).unwrap_or(0.0),
                    effective_ul: u.map(|u| u.effective_ul).unwrap_or(0.0),
                    basic_science: d.basic_science,
                    mission_injectable: d.mission_injectable,
                    dr_knee: d.dr_knee,
                }
            })
            .collect()
    }

    /// The Vehicle Designer's report (S4, FR-VD-201/501/601): a reference design
    /// derived against the FA-05 maturity the host composes for every catalogued tech
    /// (the C1 decoupling — the designer reads tech maturity, never owns it). Returns
    /// every derived figure + realism red-flag + the full parts palette. The UI
    /// computes nothing here; it only lays out the numbers.
    pub fn vehicle_report(&self) -> Option<VehicleReport> {
        // Compose the per-tech maturity map from the FA-05 surface (player faction).
        let rsnap = ResearchSnapshot::from_core(&self.core, &self.research).ok()?;
        let cat = &self.vehicle.catalogue;
        let mut maturity: BTreeMap<String, ComponentMaturity> = BTreeMap::new();
        for c in cat.components.values() {
            let cm = rsnap
                .maturity(FactionId(0), &TechId(c.tech.clone()))
                .map(|m| ComponentMaturity {
                    reliability: m.reliability,
                    trl: m.trl,
                    flyable: m.flyable,
                })
                .unwrap_or_else(ComponentMaturity::unavailable);
            maturity.insert(c.tech.clone(), cm);
        }

        // The parts palette: every catalogued component + its current maturity.
        let components: Vec<VehicleComponent> = cat
            .components
            .values()
            .map(|c| {
                let m = maturity
                    .get(&c.tech)
                    .copied()
                    .unwrap_or_else(ComponentMaturity::unavailable);
                VehicleComponent {
                    id: c.id.0.clone(),
                    class: component_class(&c.params).to_string(),
                    tech: c.tech.clone(),
                    dry_mass: c.dry_mass_kg,
                    detail: component_detail(c),
                    trl: m.trl,
                    flyable: m.flyable,
                }
            })
            .collect();

        // Derive the reference design at 1 AU.
        let mut designs = BTreeMap::new();
        designs.insert(DesignId(0), reference_design());
        let snap = DesignSnapshot::new(designs, cat.clone(), maturity);
        let out = snap.derive(VFactionId(0), DesignId(0), AU_M)?;
        let red_flags = snap
            .red_flags(DesignId(0), AU_M)
            .unwrap_or_default()
            .into_iter()
            .map(|f| RedFlagView {
                constraint: f.constraint,
                value: f.value,
                hard: matches!(f.severity, sojourn_vehicle::guards::Severity::Hard),
            })
            .collect();

        Some(VehicleReport {
            name: "Reference Probe".to_string(),
            class: "probe".to_string(),
            dry_mass: out.dry_mass,
            wet_mass: out.wet_mass,
            propellant: out.propellant,
            total_dv: out.total_dv,
            stage_dvs: out.stage_dvs,
            max_thrust: out.max_thrust,
            power_gen: out.power.generation_w,
            power_demand: out.power.demand_w,
            power_margin: out.power.margin_w,
            thermal_margin: out.thermal.margin_w,
            reliability: out.reliability,
            unit_cost: out.cost.unit_cost,
            build_days: out.cost.build_days,
            red_flags,
            components,
        })
    }

    /// The Economy & Contracts overview (S6): the player faction's conserved-currency
    /// ledger, its funding profile, the live launch market and the commodity taxonomy —
    /// all read from the FA-07 economy surface (composed inputs empty on a fresh run).
    /// The conserved-currency model means every balance traces to a real transaction.
    pub fn economy_overview(&self) -> Option<EconomyView> {
        let snap =
            EconomySnapshot::from_core(&self.core, &self.economy, EconomyInputs::default()).ok()?;
        let f = EFactionId(0);

        // The six conserved-currency balances (all zero until funding is appropriated).
        let balances = snap
            .balances(f)
            .into_iter()
            .map(|(c, amt)| CurrencyBalance {
                name: currency_label(c).to_string(),
                amount: amt,
                is_funds: matches!(c, Currency::Funds),
            })
            .collect();

        // The player's funding archetype (static profile data).
        let (funding_kind, funding_headline) = match self.economy.funding_defs.by_faction.get(&f) {
            Some(FundingProfile::Agency {
                baseline_funds,
                caretaker_floor,
                period_days,
                ..
            }) => (
                "Agency · appropriation".to_string(),
                format!(
                    "baseline {:.1} G$ / {period_days}d · caretaker floor {:.1} G$",
                    baseline_funds / 1.0e9,
                    caretaker_floor / 1.0e9
                ),
            ),
            Some(FundingProfile::Private {
                starting_cash,
                daily_burn,
                ..
            }) => (
                "Private · cash runway".to_string(),
                format!(
                    "cash {:.1} G$ · burn {:.1} M$/day",
                    starting_cash / 1.0e9,
                    daily_burn / 1.0e6
                ),
            ),
            None => ("—".to_string(), "no funding profile".to_string()),
        };

        // The launch market: $/kg per orbit class at the current world-capacity index.
        let mut launch_prices: Vec<LaunchPrice> = self
            .economy
            .launch_market
            .base_price_per_kg
            .keys()
            .filter_map(|oc| {
                snap.market_price(&oc.0).map(|p| LaunchPrice {
                    orbit_class: oc.0.clone(),
                    price_per_kg: p,
                })
            })
            .collect();
        launch_prices.sort_by(|a, b| {
            a.price_per_kg
                .partial_cmp(&b.price_per_kg)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // The tradable-commodity taxonomy.
        let commodities = self
            .economy
            .commodities
            .by_id
            .values()
            .map(|c| CommodityRow {
                id: c.id.0.clone(),
                kind: commodity_kind_label(&c.kind),
                unit: unit_label(c.unit).to_string(),
            })
            .collect();

        Some(EconomyView {
            balances,
            funding_kind,
            funding_headline,
            launch_prices,
            commodities,
        })
    }

    /// The World & Politics overview (S9): the historic-firsts race board with its
    /// claim state, the global science tide, and per-faction political state — all read
    /// from the FA-09 polity surface. The milestone catalogue is live from the first
    /// frame; factions populate once the world is initialised.
    pub fn world_overview(&self) -> Option<WorldView> {
        let snap = WorldSnapshot::from_core(&self.core, &self.polity).ok()?;

        // The full milestone catalogue with world-first claim state (the race board).
        let milestones = snap
            .ledger()
            .into_iter()
            .map(|m| MilestoneRow {
                id: m.id,
                era: m.era,
                description: m.description,
                weight: m.weight,
                claimed_by: m.world_first.map(|(f, _)| f),
            })
            .collect();

        // Per-faction political state (the agency + private archetypes, if seeded).
        let factions = [0u32, 1]
            .into_iter()
            .filter_map(|id| {
                let f = PFactionId(id);
                let mood = snap.mood(f)?;
                let modifiers = snap.modifiers(f);
                Some(FactionPolitics {
                    id,
                    mood,
                    prestige: snap.prestige(f).unwrap_or(0.0),
                    appropriation_factor: modifiers.map(|m| m.appropriation_factor).unwrap_or(1.0),
                    approval_latency_days: modifiers
                        .map(|m| m.approval_latency_days)
                        .unwrap_or(0.0),
                })
            })
            .collect();

        Some(WorldView {
            milestones,
            science_tide: snap.science_tide(),
            factions,
        })
    }

    /// The Trajectory planner's heliocentric Hohmann options (S2): a two-burn Δv + TOF
    /// from the player's home world (Terra) to each Sun-orbiting body, computed from the
    /// FA-02 catalogue's orbital radii. Real two-body physics; the UI plots the ladder.
    pub fn transfer_options(&self) -> Vec<TransferOption> {
        let snap = match AstroSnapshot::from_core(&self.core, &self.astro) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let cat = &snap.catalog;
        let Some(sun) = cat.bodies().find(|b| b.parent.is_none()) else {
            return Vec::new();
        };
        let (mu, sun_id) = (sun.mu, sun.id);
        let Some(r1) = cat
            .bodies()
            .find(|b| b.name_id == "terra")
            .and_then(|t| t.elements.map(|e| e.sma))
        else {
            return Vec::new();
        };
        if mu <= 0.0 || r1 <= 0.0 {
            return Vec::new();
        }
        let mut out: Vec<TransferOption> = Vec::new();
        for b in cat.bodies() {
            if b.parent != Some(sun_id) || b.name_id == "terra" {
                continue;
            }
            let Some(el) = b.elements else { continue };
            let r2 = el.sma;
            if r2 <= 0.0 {
                continue;
            }
            let a_t = (r1 + r2) / 2.0;
            let v1 = libm::sqrt(mu / r1);
            let vp = libm::sqrt(mu * (2.0 / r1 - 1.0 / a_t));
            let v2 = libm::sqrt(mu / r2);
            let va = libm::sqrt(mu * (2.0 / r2 - 1.0 / a_t));
            let dv = (vp - v1).abs() + (v2 - va).abs();
            let tof_s = std::f64::consts::PI * libm::sqrt(a_t * a_t * a_t / mu);
            out.push(TransferOption {
                to: display_name(&b.name_id),
                dv_mps: dv,
                tof_days: tof_s / 86_400.0,
                r_to_au: r2 / AU_M,
            });
        }
        out.sort_by(|a, b| {
            a.r_to_au
                .partial_cmp(&b.r_to_au)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    /// The Operations/Fleet roster (S5): every propagated craft from the FA-02 store,
    /// with its dominant body and flight status. Empty until craft are spawned.
    pub fn fleet(&self) -> Vec<CraftRow> {
        let snap = match AstroSnapshot::from_core(&self.core, &self.astro) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        snap.craft
            .values()
            .map(|c| CraftRow {
                id: c.id.0,
                name: display_name(&c.name_id),
                dominant: snap
                    .catalog
                    .body(c.dominant)
                    .map(|b| display_name(&b.name_id))
                    .unwrap_or_default(),
                status: format!("{:?}", c.status),
            })
            .collect()
    }

    /// The Personnel head-count by role (S8), read from the FA-05 roster. Empty (only
    /// the astronaut-ready tally) until personnel are hired.
    pub fn personnel_summary(&self) -> Vec<PersonnelRow> {
        let snap = match ResearchSnapshot::from_core(&self.core, &self.research) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        snap.personnel(FactionId(0))
            .into_iter()
            .map(|(role, count)| PersonnelRow { role, count })
            .collect()
    }

    /// The Bases & Construction catalogue (S7): the buildable module types + base-class
    /// templates from the FA-08 catalogue (live from the first frame). The UI computes
    /// nothing — power demand / mass are catalogue facts.
    pub fn base_catalogue(&self) -> BaseCatalogueView {
        let cat = &self.base.catalogue;
        let modules = cat
            .modules
            .values()
            .map(|m| BaseModuleRow {
                id: m.id.0.clone(),
                class: base_module_class(&m.params).to_string(),
                tech: m.tech.clone(),
                dry_mass: m.dry_mass_kg,
                power_demand: m.power_demand_w,
            })
            .collect();
        let classes = cat
            .classes
            .values()
            .map(|c| BaseClassRow {
                id: c.id.clone(),
                orbital: c.orbital,
                crewed: c.crewed,
            })
            .collect();
        BaseCatalogueView {
            modules,
            classes,
            founded: 0,
        }
    }

    /// The source-cited Sojournal encyclopedia (S11): every datum the game loads carries
    /// a provenance string, and the encyclopedia is simply that provenance gathered into
    /// browsable entries — the concrete embodiment of "every figure traces to a source".
    pub fn sojournal_entries(&self) -> Vec<SojournalEntry> {
        let mut out = Vec::new();
        for d in self.research.data.domains.values() {
            out.push(SojournalEntry {
                category: "Knowledge domain".to_string(),
                title: domain_name(&d.name_id),
                source: d.source.clone(),
            });
        }
        for c in self.vehicle.catalogue.components.values() {
            out.push(SojournalEntry {
                category: "Vehicle component".to_string(),
                title: c.id.0.clone(),
                source: c.source.clone(),
            });
        }
        for m in self.base.catalogue.modules.values() {
            out.push(SojournalEntry {
                category: "Base module".to_string(),
                title: m.id.0.clone(),
                source: m.source.clone(),
            });
        }
        for c in self.economy.commodities.by_id.values() {
            out.push(SojournalEntry {
                category: "Commodity".to_string(),
                title: c.id.0.clone(),
                source: c.source.clone(),
            });
        }
        for m in &self.polity.params.milestones.milestones {
            out.push(SojournalEntry {
                category: "Historic first".to_string(),
                title: m.description.clone(),
                source: m.source.clone(),
            });
        }
        out
    }

    /// The current run status (the clock every snapshot matches, FR-UI-1503).
    pub fn status(&self) -> Status {
        let s = self.core.status();
        Status {
            tick: s.tick,
            date: (s.date.year, s.date.month, s.date.day),
            total_events: 0,
            pending_interrupts: s.pending_interrupts.len() as u64,
        }
    }

    /// The most recent events (newest-first), for the feed.
    pub fn recent_events(&self, limit: u64) -> Vec<EventView> {
        let page = self.core.events(&EventFilter {
            limit,
            ..Default::default()
        });
        match page {
            Ok(p) => p
                .events
                .into_iter()
                .map(|e| EventView {
                    id: e.id.0,
                    tick: e.tick,
                    class: e.class,
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Advance the core by `ticks` (the warp-appropriate stride); returns the number
    /// of interrupts now pending. Never blocks beyond this call (FR-UI-1503).
    pub fn advance(&mut self, ticks: u64) -> Result<u64, CoreError> {
        self.core.step(StepRequest::Ticks(ticks))?;
        Ok(self.core.status().pending_interrupts.len() as u64)
    }

    /// The pending interrupts `(id, class)` — what the world paused on (FR-UI-1003).
    pub fn pending_interrupts(&self) -> Vec<(u64, String)> {
        self.core
            .status()
            .pending_interrupts
            .iter()
            .map(|i| (i.id, i.class.clone()))
            .collect()
    }

    /// Acknowledge a pending interrupt (resume).
    pub fn acknowledge(&mut self, id: u64) -> Result<(), CoreError> {
        self.core.acknowledge(id)?;
        Ok(())
    }

    /// Acknowledge **all** pending interrupts (and any cascade they raise), resuming
    /// the interrupt-and-pause loop. Bounded against a runaway cascade.
    pub fn acknowledge_all(&mut self) -> Result<(), CoreError> {
        for _ in 0..256 {
            let ids: Vec<u64> = self
                .core
                .status()
                .pending_interrupts
                .iter()
                .map(|i| i.id)
                .collect();
            if ids.is_empty() {
                break;
            }
            for id in ids {
                self.core.acknowledge(id)?;
            }
            self.core.step(StepRequest::Ticks(0))?;
        }
        Ok(())
    }

    /// Submit a typed journalled command; surface the outcome (rejection reason
    /// included, never hidden — FR-UI-305).
    pub fn submit(&mut self, cmd: Command) -> CommitOutcome {
        match self.core.submit(cmd) {
            Ok(_) => {
                // apply the command (zero-tick step) so its outcome is realised.
                match self.core.step(StepRequest::Ticks(0)) {
                    Ok(_) => CommitOutcome::Applied,
                    Err(e) => CommitOutcome::Rejected(e.to_string()),
                }
            }
            Err(e) => CommitOutcome::Rejected(e.to_string()),
        }
    }
}

/// Map a core `CommandOutcome` to the UI `CommitOutcome`.
pub fn outcome(o: CommandOutcome) -> CommitOutcome {
    match o {
        CommandOutcome::Applied => CommitOutcome::Applied,
        CommandOutcome::Rejected(r) => CommitOutcome::Rejected(r),
    }
}
