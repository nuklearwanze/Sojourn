//! US6 — base state exposed to economy, life support & politics.

mod common;
use common::*;
use sojourn_base::BaseCommand;
use sojourn_base::ids::BaseId;

#[test]
fn production_consumption_and_life_support_are_exposed() {
    let mut e = BaseH::new();
    let bid = e.build(
        "luna",
        "surface-base",
        &["pwr-fission", "hab-rigid", "eclss-physchem", "fab-shop"],
    );
    let s = e.snap(inputs_for("luna", benign_site()));

    // Production/consumption for the economy: produces spares, consumes consumables.
    let pc = s.production_consumption(bid).unwrap();
    assert!(
        pc.produces.get("spares").copied().unwrap_or(0.0) > 0.0,
        "produces spares"
    );
    assert!(
        pc.consumes.get("consumables").copied().unwrap_or(0.0) > 0.0,
        "consumes consumables"
    );

    // Habitat state for life support (Slice 8).
    let ls = s.life_support(bid).unwrap();
    assert_eq!(ls.population_capacity, 4);
    assert!(ls.closure_fraction > 0.0);
}

#[test]
fn embargo_survival_records_a_settlement_milestone() {
    let mut e = BaseH::new();
    let _ = e.build(
        "luna",
        "settlement",
        &["pwr-fission", "hab-rigid", "eclss-highclosure"],
    );
    e.cmd(BaseCommand::EvaluateEmbargo {
        base: 0,
        years: 5.0,
        survives: true,
    });
    let s = e.snap(inputs_for("luna", benign_site()));
    assert!(
        s.milestones()
            .iter()
            .any(|m| m.contains("embargo-survivor")),
        "milestone recorded: {:?}",
        s.milestones()
    );
}

#[test]
fn compare_diffs_two_bases() {
    let mut e = BaseH::new();
    e.cmd(BaseCommand::FoundBase {
        faction: 0,
        site: "luna".into(),
        class: "surface-base".into(),
    });
    e.cmd(BaseCommand::AddModule {
        base: 0,
        module_type: "pwr-fission".into(),
    });
    e.cmd(BaseCommand::AddModule {
        base: 0,
        module_type: "hab-rigid".into(),
    });
    e.cmd(BaseCommand::OpenConstruction { base: 0 });
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 0,
        mass_kg: 1e7,
        crew_time_hr: 1e6,
    });
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 1,
        mass_kg: 1e7,
        crew_time_hr: 1e6,
    });
    // A second, smaller base.
    e.cmd(BaseCommand::FoundBase {
        faction: 0,
        site: "luna".into(),
        class: "surface-base".into(),
    });
    e.cmd(BaseCommand::AddModule {
        base: 1,
        module_type: "pwr-solar".into(),
    });
    e.cmd(BaseCommand::OpenConstruction { base: 1 });
    e.cmd(BaseCommand::DeliverToBase {
        base: 1,
        module: 0,
        mass_kg: 1e7,
        crew_time_hr: 1e6,
    });

    let s = e.snap(inputs_for("luna", benign_site()));
    let (a, b) = s.compare(BaseId(0), BaseId(1)).unwrap();
    assert!(
        a.power.demand_w > b.power.demand_w,
        "base 0 has more demand (a habitat)"
    );
}
