//! US2 — construction projects routed through logistics (delivery-driven commissioning).

mod common;
use common::*;
use sojourn_base::BaseCommand;
use sojourn_base::ids::{BaseId, ModuleId};

#[test]
fn modules_commission_only_when_mass_and_crew_time_land() {
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

    // Deliver module 0 (power) fully → commissions; module 1 untouched.
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 0,
        mass_kg: 1.0e7,
        crew_time_hr: 1.0e6,
    });
    let s = e.snap(inputs_for("luna", benign_site()));
    let b = s.base(BaseId(0)).unwrap();
    assert!(b.modules[&ModuleId(0)].commissioned, "power commissioned");
    assert!(!b.modules[&ModuleId(1)].commissioned, "habitat not yet");
    // Partial base ⇒ only the power module contributes: gen 40 kW, demand 0.
    assert!((s.power(BaseId(0)).unwrap().margin_w - 40_000.0).abs() < 1.0);
    let prog = s.construction(BaseId(0)).unwrap();
    assert_eq!((prog.commissioned_count, prog.module_count), (1, 2));

    // Deliver module 1 → commissions; demand now includes the habitat.
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 1,
        mass_kg: 1.0e7,
        crew_time_hr: 1.0e6,
    });
    let s = e.snap(inputs_for("luna", benign_site()));
    assert!(
        (s.power(BaseId(0)).unwrap().margin_w - 37_500.0).abs() < 1.0,
        "habitat demand now counted"
    );
    assert_eq!(s.construction(BaseId(0)).unwrap().commissioned_count, 2);
}

#[test]
fn insufficient_crew_time_blocks_commissioning() {
    let mut e = BaseH::new();
    e.cmd(BaseCommand::FoundBase {
        faction: 0,
        site: "luna".into(),
        class: "surface-base".into(),
    });
    e.cmd(BaseCommand::AddModule {
        base: 0,
        module_type: "hab-rigid".into(),
    });
    e.cmd(BaseCommand::OpenConstruction { base: 0 });
    // hab-rigid: dry_mass 12000 → required crew-time 12000 × 0.02 = 240 hr.
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 0,
        mass_kg: 12_000.0,
        crew_time_hr: 100.0,
    });
    assert!(
        !e.snap(inputs_for("luna", benign_site()))
            .base(BaseId(0))
            .unwrap()
            .modules[&ModuleId(0)]
            .commissioned,
        "insufficient crew-time blocks commissioning"
    );
    // Top up crew-time past the threshold → commissions.
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 0,
        mass_kg: 0.0,
        crew_time_hr: 200.0,
    });
    assert!(
        e.snap(inputs_for("luna", benign_site()))
            .base(BaseId(0))
            .unwrap()
            .modules[&ModuleId(0)]
            .commissioned,
        "crew-time now sufficient"
    );
}
