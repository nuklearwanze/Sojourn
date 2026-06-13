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
                        bail!("catalogue body '{}' uses a reserved generated id", b.name_id);
                    }
                }
                // Major-body Sojournal coverage: Sun, Earth, Mars (curated subset).
                let majors: Vec<sojourn_astro::BodyId> =
                    [0u32, 3, 4].iter().map(|&i| sojourn_astro::BodyId(i)).collect();
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
                    m.world_hash.iter().map(|b| format!("{b:02x}")).collect::<String>()
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
                    if !sojourn_research::seeding::every_category_reachable(&module.data, &dead_ends) {
                        bricked += 1;
                    }
                }
                if bricked > 0 {
                    bail!("reachability sweep FAILED: {bricked}/200 seeds bricked a capability category");
                }
                println!(
                    "DATA VALID (research): {} domains, {} tech nodes, {} categories, {} traits; \
                     reachability sweep PASS (200 seeds); research hash {}",
                    module.data.domains.len(),
                    module.data.nodes.len(),
                    module.data.categories.len(),
                    module.data.traits.len(),
                    module.data.content_hash.iter().map(|b| format!("{b:02x}")).collect::<String>()
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
                other => bail!(
                    "unknown module '{other}' (use 'toy', 'synthetic', 'astro', 'world' or 'research')"
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
