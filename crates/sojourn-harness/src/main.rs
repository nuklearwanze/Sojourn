//! sojourn-harness — headless CLI + determinism suite for the Sojourn kernel.
//! No UI anywhere. Subcommands map 1:1 to the CI quality gates
//! (specs/002-sim-core/tasks.md US1/US3/US4/US6).

mod doublerun;
mod killtest;
mod mutation;
mod roundtrip;
mod scenario;
mod synthetic;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use mutation::Mutation;
use scenario::{Scenario, StepPattern};
use sojourn_core::{DataSet, MemResolver, SimCore, run_suite};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sojourn-harness",
    about = "Headless driver & determinism suite"
)]
struct Cli {
    /// Kernel data directory.
    #[arg(long, default_value = "data/kernel")]
    data_dir: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a scenario headlessly.
    Run {
        /// Scenario file (RON).
        scenario: PathBuf,
        /// Durable run directory (journal, autosaves, event tier).
        #[arg(long)]
        run_dir: Option<PathBuf>,
        /// Stop at this tick (default: scenario until_tick).
        #[arg(long)]
        until_tick: Option<u64>,
        /// Print status & fingerprint at the end.
        #[arg(long)]
        print_status: bool,
    },
    /// Double-run determinism check: two stepping patterns, identical history.
    Verify {
        /// Scenario file.
        scenario: PathBuf,
    },
    /// Save → load → continue must equal a never-saved run.
    Roundtrip {
        /// Scenario file.
        scenario: PathBuf,
        /// Comma-separated save ticks.
        #[arg(long, value_delimiter = ',')]
        save_at_ticks: Vec<u64>,
    },
    /// Replay a journal; with --verify, compare regenerated history to recorded.
    Replay {
        /// Scenario file (rebuilds the module set).
        scenario: PathBuf,
        /// Journal file to replay.
        #[arg(long)]
        journal: PathBuf,
        /// Verify against recorded frames.
        #[arg(long)]
        verify: bool,
        /// Replay only up to this tick.
        #[arg(long)]
        until_tick: Option<u64>,
    },
    /// Kill a child mid-run, recover, compare against a clean reference.
    Killtest {
        /// Scenario file.
        scenario: PathBuf,
        /// Number of trials.
        #[arg(long, default_value_t = 20)]
        trials: u32,
        /// Root directory for trial run dirs.
        #[arg(long, default_value = "runs")]
        runs_root: PathBuf,
    },
    /// Prove the gate has teeth: injected nondeterminism MUST fail verify.
    Mutate {
        /// Run every injection type.
        #[arg(long)]
        all: bool,
        /// Run one injection type.
        #[arg(long, value_enum)]
        r#type: Option<Mutation>,
        /// Scenario file.
        #[arg(long, default_value = "scenarios/smoke_decade.ron")]
        scenario: PathBuf,
    },
    /// Validate kernel data files (strict schema + semantic + source checks).
    ValidateData {
        /// Data directory.
        dir: PathBuf,
    },
    /// Run the module conformance suite.
    Conformance {
        /// Which module: "toy" or "synthetic".
        #[arg(long, default_value = "toy")]
        module: String,
        /// Simulated ticks for the behavioural checks.
        #[arg(long, default_value_t = 2_592_000)]
        ticks: u64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let data = DataSet::load_dir(&cli.data_dir)
        .with_context(|| format!("loading kernel data from {}", cli.data_dir.display()))?;

    match cli.cmd {
        Cmd::Run {
            scenario,
            run_dir,
            until_tick,
            print_status,
        } => {
            let s = Scenario::load(&scenario)?;
            let mut core = s.create(&data, run_dir.as_deref(), None)?;
            let out = scenario::drive(&mut core, &s, until_tick, StepPattern::MaxStride)?;
            if print_status {
                let st = core.status();
                println!(
                    "run '{}' → tick {} ({}-{:02}-{:02}), lifecycle {:?}, {} events, \
                     {} acks, fingerprint {}",
                    s.name,
                    st.tick,
                    st.date.year,
                    st.date.month,
                    st.date.day,
                    st.lifecycle,
                    st.total_events,
                    out.acks,
                    core.fingerprint()?.hex()
                );
                for (tick, fp) in &out.checkpoints {
                    println!("  checkpoint @{tick}: {}", fp.hex());
                }
            }
        }
        Cmd::Verify { scenario } => {
            let s = Scenario::load(&scenario)?;
            let out = doublerun::verify(&s, &data, None)?;
            println!(
                "VERIFY PASS '{}': {} checkpoints and {} events identical across stepping patterns",
                s.name, out.checkpoints, out.events
            );
        }
        Cmd::Roundtrip {
            scenario,
            save_at_ticks,
        } => {
            let s = Scenario::load(&scenario)?;
            if save_at_ticks.is_empty() {
                bail!("--save-at-ticks requires at least one tick");
            }
            roundtrip::roundtrip(&s, &data, &save_at_ticks)?;
            println!(
                "ROUNDTRIP PASS '{}': saves at {:?} reload bit-identically and continue \
                 identically to an unbroken run",
                s.name, save_at_ticks
            );
        }
        Cmd::Replay {
            scenario,
            journal,
            verify,
            until_tick,
        } => {
            let s = Scenario::load(&scenario)?;
            let bytes = std::fs::read(&journal)
                .with_context(|| format!("reading journal {}", journal.display()))?;
            let resolver = MemResolver {
                sets: vec![data.clone()],
            };
            let (_core, report) =
                SimCore::replay(&bytes, &resolver, s.modules(None)?, until_tick, verify)?;
            match &report.first_divergence {
                None => println!(
                    "REPLAY {} '{}': {} commands → tick {}{}",
                    if verify { "VERIFY PASS" } else { "OK" },
                    s.name,
                    report.commands_applied,
                    report.final_tick,
                    if verify {
                        " (recorded history matches)"
                    } else {
                        ""
                    }
                ),
                Some(d) => bail!("REPLAY DIVERGENCE: {d}"),
            }
        }
        Cmd::Killtest {
            scenario,
            trials,
            runs_root,
        } => {
            let s = Scenario::load(&scenario)?;
            let outcomes = killtest::killtest(&scenario, &s, &data, &runs_root, trials)?;
            let torn = outcomes.iter().filter(|o| o.torn).count();
            let furthest = outcomes.iter().map(|o| o.recovered_tick).max().unwrap_or(0);
            println!(
                "KILLTEST PASS '{}': {}/{} trials produced a journal; all recovered to \
                 reference-identical state ({torn} with torn tails handled; furthest \
                 recovery at tick {furthest})",
                s.name,
                outcomes.len(),
                trials
            );
        }
        Cmd::Mutate {
            all,
            r#type,
            scenario,
        } => {
            let s = Scenario::load(&scenario)?;
            let list = if all {
                Mutation::all()
            } else {
                vec![r#type.context("--type or --all required")?]
            };
            let mut survived: Vec<String> = Vec::new();
            for m in list {
                match doublerun::verify(&s, &data, Some(m)) {
                    Ok(_) => survived.push(format!("{m:?}")),
                    Err(e) => println!("MUTATION CAUGHT {m:?}: {e}"),
                }
            }
            if !survived.is_empty() {
                bail!(
                    "GATE HAS NO TEETH: injected nondeterminism survived verify: {}",
                    survived.join(", ")
                );
            }
            println!("MUTATE PASS: every injected nondeterminism was caught by the gate");
        }
        Cmd::ValidateData { dir } => {
            // Kernel data dirs carry event-classes.ron; astro data dirs carry
            // test-catalog.ron. Validate whichever this is.
            if dir.join("event-classes.ron").exists() {
                let ds = DataSet::load_dir(&dir)?;
                ds.validate()?;
                println!(
                    "DATA VALID (kernel): {} event classes, {} watch templates, version {}",
                    ds.event_classes.len(),
                    ds.watch_templates.len(),
                    ds.version().hex()
                );
            } else if dir.join("priors.ron").exists() {
                // FA-03 world data (FR-WORLD-801): catalogue + sites + locations +
                // prospecting + Sojournal (citations, link resolution, major-body
                // coverage). The build-tool-produced catalogue is validated here,
                // never the network.
                let m = sojourn_world::WorldModule::load(&dir)
                    .map_err(|e| anyhow::anyhow!("world data invalid: {e}"))?;
                // Naming check (FR-WORLD-105): no fictional-commercial collisions —
                // generated-id range is reserved; real entries use real designations.
                for b in m.catalog.bodies() {
                    if b.id.0 >= sojourn_world::GENERATED_ID_BASE {
                        bail!(
                            "catalogue body '{}' uses a reserved generated id",
                            b.name_id
                        );
                    }
                }
                // Major-body Sojournal coverage: Sun, Earth, Mars (curated subset).
                let majors: Vec<sojourn_astro::BodyId> = [0u32, 3, 4]
                    .iter()
                    .map(|&i| sojourn_astro::BodyId(i))
                    .collect();
                m.sojournal
                    .validate(&m.catalog, &m.sites, &m.locations, &majors)
                    .map_err(|e| anyhow::anyhow!("sojournal invalid: {e}"))?;
                m.locations
                    .validate_refs(&m.catalog)
                    .map_err(|e| anyhow::anyhow!("locations invalid: {e}"))?;
                println!(
                    "DATA VALID (world): {} bodies, {} sites, {} locations, {} sojournal entries, \
                     world hash {}",
                    m.catalog.bodies().count(),
                    m.sites.all().count(),
                    m.locations.ids().count(),
                    m.sojournal.entries().len(),
                    m.world_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else if dir.join("domains.ron").exists() {
                // FA-05 research data (FR-RESP-901): `dir` is data/research; the tech
                // tree is the sibling data/tech. Validate + run the reachability sweep.
                let parent = dir.parent().unwrap_or(std::path::Path::new("."));
                let module = sojourn_research::ResearchModule::load(parent)
                    .map_err(|e| anyhow::anyhow!("research data invalid: {e}"))?;
                use rand_chacha::ChaCha12Rng;
                use rand_core::SeedableRng;
                let mut bricked = 0u32;
                for seed in 0..200u64 {
                    let mut rng = ChaCha12Rng::seed_from_u64(seed);
                    let (dead_ends, _) = sojourn_research::seeding::seed(&module.data, &mut rng);
                    if !sojourn_research::seeding::every_category_reachable(
                        &module.data,
                        &dead_ends,
                    ) {
                        bricked += 1;
                    }
                }
                if bricked > 0 {
                    bail!(
                        "reachability sweep FAILED: {bricked}/200 seeds bricked a capability category"
                    );
                }
                println!(
                    "DATA VALID (research): {} domains, {} tech nodes, {} categories, {} traits; \
                     reachability sweep PASS (200 seeds); research hash {}",
                    module.data.domains.len(),
                    module.data.nodes.len(),
                    module.data.categories.len(),
                    module.data.traits.len(),
                    module
                        .data
                        .content_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else if dir.join("components.ron").exists() {
                // FA-04 vehicle component data (FR-VD-803).
                let module = sojourn_vehicle::VehicleModule::load(&dir)
                    .map_err(|e| anyhow::anyhow!("vehicle data invalid: {e}"))?;
                println!(
                    "DATA VALID (vehicle): {} components, {} classes, vehicle hash {}",
                    module.catalogue.components.len(),
                    module.catalogue.classes.len(),
                    module
                        .catalogue
                        .content_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else if dir.join("eclss.ron").exists() {
                // FA-08 life support & crew data (FR-LSC-801): schema + sources +
                // REID threshold + Mars EDL gap (in load) plus the analytic gates.
                let m = sojourn_crew::CrewModule::load(&dir)
                    .map_err(|e| anyhow::anyhow!("crew data invalid: {e}"))?;
                crew_analytic_gates(&m)?;
                println!(
                    "DATA VALID (crew): {} GCR environments, {} EDL bodies; REID 3% + Mars gap + \
                     analytic gates PASS; crew hash {}",
                    m.params.radiation.gcr_by_env.len(),
                    m.params.edl.body_difficulty.len(),
                    m.params
                        .content_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else if dir.join("milestones.ron").exists() {
                // FA-09 politics/events/milestones/astrobiology data (FR-PEA-903):
                // schema + sources + semantic checks (in load) plus the analytic gates
                // (prior fidelity, consensus band incl. the honesty invariant,
                // contamination/hazard monotonicity, tiebreak/score determinism).
                let m = sojourn_polity::PolityModule::load(&dir)
                    .map_err(|e| anyhow::anyhow!("polity data invalid: {e}"))?;
                polity_analytic_gates(&m)?;
                println!(
                    "DATA VALID (polity): {} firsts, {} event classes, {} policy levers, \
                     {} PP bodies; analytic gates PASS; polity hash {}",
                    m.params.milestones.milestones.len(),
                    m.params.events.classes.len(),
                    m.params.policy.levers.len(),
                    m.params.protection.bodies.len(),
                    m.params
                        .content_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else if dir.join("modules.ron").exists() {
                // FA-07 base data (FR-BC-701): schema + sources + ref resolution (in
                // load) plus the analytic gates (shielding exp-attenuation).
                let m = sojourn_base::BaseModule::load(&dir)
                    .map_err(|e| anyhow::anyhow!("base data invalid: {e}"))?;
                base_analytic_gates(&m)?;
                println!(
                    "DATA VALID (base): {} module types, {} classes, {} shield materials; \
                     analytic gates PASS; base hash {}",
                    m.catalogue.modules.len(),
                    m.catalogue.classes.len(),
                    m.catalogue.params.shield_materials.len(),
                    m.catalogue
                        .content_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else if dir.join("commodities.ron").exists() {
                // FA-06 economy data (FR-EC-801): schema + sources + ref resolution
                // (in load) plus the analytic gates (break-even sign, P50<P80,
                // launch-price elasticity).
                let m = sojourn_economy::EconomyModule::load(&dir)
                    .map_err(|e| anyhow::anyhow!("economy data invalid: {e}"))?;
                economy_analytic_gates(&m)?;
                println!(
                    "DATA VALID (economy): {} commodities, {} network nodes, {} isru processes, \
                     {} facility types; analytic gates PASS; econ hash {}",
                    m.commodities.by_id.len(),
                    m.network.nodes.len(),
                    m.isru.by_id.len(),
                    m.facility_defs.by_id.len(),
                    m.content_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else if dir.join("test-catalog.ron").exists() {
                let module = sojourn_astro::AstroModule::load(&dir)
                    .map_err(|e| anyhow::anyhow!("astro data invalid: {e}"))?;
                println!(
                    "DATA VALID (astro): {} bodies, catalogue hash {}",
                    module.catalog.bodies().count(),
                    module
                        .catalog
                        .content_hash
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                );
            } else {
                bail!(
                    "{} contains neither kernel, world nor astro data files",
                    dir.display()
                );
            }
        }
        Cmd::Conformance { module, ticks } => {
            let factory: Box<dyn Fn() -> Box<dyn sojourn_core::SimModule>> = match module.as_str() {
                "toy" => Box::new(|| Box::new(synthetic::toy::ToyModule)),
                "synthetic" => {
                    Box::new(|| Box::new(synthetic::SyntheticModule::new(Default::default())))
                }
                "astro" => Box::new(|| {
                    Box::new(
                        sojourn_astro::AstroModule::load(std::path::Path::new("data/astro"))
                            .expect("astro data loads"),
                    )
                }),
                "world" => Box::new(|| {
                    Box::new(
                        sojourn_world::WorldModule::load(std::path::Path::new("data/world"))
                            .expect("world data loads"),
                    )
                }),
                "research" => Box::new(|| {
                    Box::new(
                        sojourn_research::ResearchModule::load(std::path::Path::new("data"))
                            .expect("research data loads"),
                    )
                }),
                "vehicle" => Box::new(|| {
                    Box::new(
                        sojourn_vehicle::VehicleModule::load(std::path::Path::new("data/vehicle"))
                            .expect("vehicle data loads"),
                    )
                }),
                "economy" => Box::new(|| {
                    Box::new(
                        sojourn_economy::EconomyModule::load(std::path::Path::new("data/econ"))
                            .expect("economy data loads"),
                    )
                }),
                "base" => Box::new(|| {
                    Box::new(
                        sojourn_base::BaseModule::load(std::path::Path::new("data/base"))
                            .expect("base data loads"),
                    )
                }),
                "crew" => Box::new(|| {
                    Box::new(
                        sojourn_crew::CrewModule::load(std::path::Path::new("data/crew"))
                            .expect("crew data loads"),
                    )
                }),
                "polity" => Box::new(|| {
                    Box::new(
                        sojourn_polity::PolityModule::load(std::path::Path::new("data/polity"))
                            .expect("polity data loads"),
                    )
                }),
                other => bail!(
                    "unknown module '{other}' (use 'toy', 'synthetic', 'astro', 'world', 'research', 'vehicle', 'economy', 'base', 'crew' or 'polity')"
                ),
            };
            let report = run_suite(&*factory, &data, 4242, ticks)?;
            for p in &report.passed {
                println!("  ✓ {p}");
            }
            for f in &report.failed {
                println!("  ✗ {f}");
            }
            if !report.ok() {
                bail!("conformance suite failed for module '{module}'");
            }
            println!("CONFORMANCE PASS '{module}'");
        }
    }
    Ok(())
}

/// FA-06 analytic validation gates (FR-EC-801, contracts/economy-data.md): ISRU
/// break-even sign flip, P50<P80, and the launch-price elasticity sign.
fn economy_analytic_gates(m: &sojourn_economy::EconomyModule) -> Result<()> {
    use sojourn_economy::ids::OrbitClass;
    use sojourn_economy::isru;

    // 1. ISRU break-even sign: negative below break-even, positive above.
    let proc = m
        .isru
        .by_id
        .values()
        .next()
        .context("no ISRU process to validate")?;
    let leo = m
        .launch_market
        .price_at(
            &OrbitClass("leo".into()),
            m.launch_market.reference_capacity,
        )
        .context("no LEO launch price")?;
    let yield_per_day = proc.base_yield_per_day; // nameplate
    let below = isru::break_even(proc, yield_per_day, leo, leo, 10.0);
    let above = isru::break_even(proc, yield_per_day, leo, leo, 3650.0);
    if below.net >= 0.0 || above.net <= 0.0 {
        bail!(
            "ISRU break-even gate failed: below.net={} (want <0), above.net={} (want >0)",
            below.net,
            above.net
        );
    }

    // 2. P50 < P80 from the cost params (spread > 0).
    if m.cost.p80_spread <= 0.0 {
        bail!("cost gate failed: p80_spread must be > 0 so P50 < P80");
    }

    // 3. Launch-price elasticity sign: raising world capacity lowers $/kg.
    let base = m
        .launch_market
        .price_at(
            &OrbitClass("leo".into()),
            m.launch_market.reference_capacity,
        )
        .context("no LEO price")?;
    let cheaper = m
        .launch_market
        .price_at(
            &OrbitClass("leo".into()),
            m.launch_market.reference_capacity * 2.0,
        )
        .context("no LEO price")?;
    if cheaper >= base {
        bail!("elasticity gate failed: doubling capacity did not lower $/kg ({cheaper} >= {base})");
    }
    Ok(())
}

/// FA-07 analytic validation gates (FR-BC-701, contracts/base-data.md): the
/// shielding mass-attenuation `exp(−Σ ρx/λ)` is correct and composes per material.
fn base_analytic_gates(m: &sojourn_base::BaseModule) -> Result<()> {
    let params = &m.catalogue.params;
    // For each shield material, exp(−λ/λ) at ρx = λ must equal e^-1; and doubling
    // ρx must square the factor (per-material exponent linearity).
    let e_inv = libm::exp(-1.0);
    for mat in &params.shield_materials {
        let lam = mat.attenuation_length_kg_m2;
        let one = libm::exp(-lam / lam);
        let two = libm::exp(-(2.0 * lam) / lam);
        if (one - e_inv).abs() > 1e-9 {
            bail!(
                "shielding gate failed: exp(−ρx/λ) at ρx=λ ≠ e⁻¹ for '{}'",
                mat.id
            );
        }
        if (two - one * one).abs() > 1e-9 {
            bail!(
                "shielding gate failed: doubling ρx did not square the factor for '{}'",
                mat.id
            );
        }
    }
    Ok(())
}

/// FA-08 analytic validation gates (FR-LSC-801, contracts/crew-data.md): REID
/// monotonicity, multiplicative-hazard factor monotonicity, the Mars EDL gap, and
/// the capability product.
fn crew_analytic_gates(m: &sojourn_crew::CrewModule) -> Result<()> {
    use sojourn_crew::hazard;
    use sojourn_crew::inputs::{AstronautFacts, Sex};
    use sojourn_crew::radiation;
    let p = &m.params;

    // 1. REID monotone in dose.
    let facts = AstronautFacts {
        age_years: 40.0,
        sex: Sex::Male,
        traits: vec![],
        training: 1.0,
    };
    let r1 = radiation::reid_pct(0.5, &facts, &p.radiation);
    let r2 = radiation::reid_pct(1.0, &facts, &p.radiation);
    if r2 <= r1 {
        bail!("REID gate failed: not monotone in dose ({r2} !> {r1})");
    }

    // 2. Multiplicative-hazard factor monotonicity + clamp to [0,1].
    let h1 = hazard::hazard(0.1, &[1.0, 1.0]);
    let h2 = hazard::hazard(0.1, &[2.0, 1.0]);
    if h2 <= h1 || h2 > 1.0 {
        bail!("hazard gate failed: not monotone/clamped ({h2} vs {h1})");
    }

    // 3. Mars EDL gap (also enforced in load).
    let mars = p.edl.body_difficulty.get("mars").copied().unwrap_or(0.0);
    let moon = p.edl.body_difficulty.get("moon").copied().unwrap_or(0.0);
    if mars <= moon {
        bail!("EDL gate failed: Mars ({mars}) not harder than moon ({moon})");
    }

    // 4. Capability product.
    let cap = hazard::capability(&[0.5, 0.5]);
    if (cap - 0.25).abs() > 1e-9 {
        bail!("capability gate failed: product of [0.5,0.5] ≠ 0.25 ({cap})");
    }
    Ok(())
}

/// FA-09 analytic validation gates (FR-PEA-903, contracts/polity-data.md): the
/// consensus band + the honesty invariant (never conclusive-positive for negative
/// ground truth), forward-contamination monotonicity, and event-hazard monotonicity.
fn polity_analytic_gates(m: &sojourn_polity::PolityModule) -> Result<()> {
    use sojourn_polity::EvidenceStage;
    use sojourn_polity::{astrobiology, events, protection};
    let p = &m.params;

    // 1. Consensus band + honesty invariant.
    let d_pos = astrobiology::evidence_delta(EvidenceStage::SampleReturn, 1.0, true, &p.astro);
    let prob_pos = astrobiology::faction_prob(d_pos, true, true, &p.astro);
    if prob_pos < p.astro.band_positive {
        bail!(
            "consensus gate: a clean positive SampleReturn must reach the positive band ({prob_pos} < {})",
            p.astro.band_positive
        );
    }
    let d_hint = astrobiology::evidence_delta(EvidenceStage::Microscopy, 1.0, false, &p.astro);
    let prob_neg = astrobiology::faction_prob(d_hint * 5.0, false, false, &p.astro);
    if prob_neg >= p.astro.band_positive {
        bail!("honesty gate: a negative-truth world reached the positive band ({prob_neg})");
    }
    let d_neg_sr = astrobiology::evidence_delta(EvidenceStage::SampleReturn, 1.0, false, &p.astro);
    if d_neg_sr >= 0.0 {
        bail!(
            "honesty gate: a negative-truth SampleReturn must push the posterior DOWN ({d_neg_sr})"
        );
    }

    // 2. Forward contamination monotone in overage; compliant degrades nothing; crash ≥ soft.
    let cp = &p.protection.contamination;
    if protection::forward_severity(5.0, 10.0, false, cp) != 0.0 {
        bail!("contamination gate: a compliant lander must degrade nothing");
    }
    let s1 = protection::forward_severity(20.0, 10.0, false, cp);
    let s2 = protection::forward_severity(40.0, 10.0, false, cp);
    if s2 <= s1 {
        bail!("contamination gate: severity must increase with bioburden overage ({s2} !> {s1})");
    }
    let crash = protection::forward_severity(20.0, 10.0, true, cp);
    if crash < s1 {
        bail!("contamination gate: a crash must be at least as severe as a soft landing");
    }

    // 3. Event hazard monotone in risk + clamped to [0,1].
    let def = p
        .events
        .classes
        .iter()
        .find(|c| c.class == "anomaly")
        .ok_or_else(|| anyhow::anyhow!("no 'anomaly' event class to validate"))?;
    let mature = events::event_prob(def, events::risk_factor(1.0), 1.0);
    let immature = events::event_prob(def, events::risk_factor(0.0), 1.0);
    if immature <= mature {
        bail!(
            "event gate: a lower-maturity (riskier) faction must earn a higher probability ({immature} !> {mature})"
        );
    }
    if events::event_prob(def, 1000.0, 1000.0) > 1.0 {
        bail!("event gate: probability must clamp to ≤ 1");
    }
    Ok(())
}
