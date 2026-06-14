//! US4 — on-site ISRU & fabrication reduce imports.

mod common;
use common::*;
use sojourn_base::BaseCommand;
use sojourn_base::ids::BaseId;
use sojourn_base::inputs::IsruOutput;

#[test]
fn regolith_built_shielding_avoids_imported_mass() {
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
        module_type: "shield-regolith".into(),
    });
    e.cmd(BaseCommand::OpenConstruction { base: 0 });
    // Power delivered conventionally; shielding built from local regolith (no import).
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 0,
        mass_kg: 1.0e7,
        crew_time_hr: 1.0e6,
    });
    e.cmd(BaseCommand::BuildLocal {
        base: 0,
        module: 1,
        local_mass_kg: 1.0e7,
    });
    // Crew-time still needed to commission the locally-built module.
    e.cmd(BaseCommand::DeliverToBase {
        base: 0,
        module: 1,
        mass_kg: 0.0,
        crew_time_hr: 1.0e6,
    });

    // Compose ISRU regolith output so on-site shielding production is non-zero.
    let mut inputs = inputs_for("luna", benign_site());
    inputs.isru.insert(
        (BaseId(0), "regolith".into()),
        IsruOutput {
            rate_kg_per_day: 100.0,
        },
    );
    let s = e.snap(inputs);

    let b = s.base(BaseId(0)).unwrap();
    assert!(
        b.modules[&sojourn_base::ids::ModuleId(1)].built_local,
        "shielding built locally"
    );
    let lp = s.local_production(BaseId(0)).unwrap();
    // shield-regolith dry mass (500 kg) avoided as imported mass.
    assert!(
        (lp.import_mass_avoided_kg - 500.0).abs() < 1e-6,
        "import mass avoided: {}",
        lp.import_mass_avoided_kg
    );
    // 100 kg/day regolith × 0.8 ratio = 80 kg/day shielding produced on-site.
    assert!(
        (lp.regolith_shield_rate_kg_day - 80.0).abs() < 1e-6,
        "regolith shield rate: {}",
        lp.regolith_shield_rate_kg_day
    );

    // The locally-built module's mass does not count against the imported-mass remainder.
    let prog = s.construction(BaseId(0)).unwrap();
    assert_eq!(prog.commissioned_count, 2, "both modules operational");
}
