//! Shared fixtures for the vehicle integration tests: the catalogue, design
//! builders, a stub maturity provider, and snapshot construction.
#![allow(dead_code)]

use sojourn_vehicle::catalogue::Catalogue;
use sojourn_vehicle::design::{MissionReqs, RedundancyBlock, Stage, VehicleDesign};
use sojourn_vehicle::{
    ComponentId, ComponentMaturity, DesignId, DesignSnapshot, FactionId, VehicleModule,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 1 AU (m).
pub const AU: f64 = 1.495_978_707e11;

pub fn repo(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(p)
}

pub fn vehicle_module() -> VehicleModule {
    VehicleModule::load(&repo("data/vehicle")).expect("vehicle data")
}

/// A stub maturity map: every component technology → the given reliability/TRL.
pub fn maturity_for(
    m: &VehicleModule,
    reliability: f64,
    trl: u8,
) -> BTreeMap<String, ComponentMaturity> {
    let mut map = BTreeMap::new();
    for c in m.catalogue.components.values() {
        map.insert(
            c.tech.clone(),
            ComponentMaturity {
                reliability,
                trl,
                flyable: trl >= 6,
            },
        );
    }
    map
}

/// A stage from component ids + propellant + engine id.
pub fn stage(components: &[&str], propellant: f64, engine: Option<&str>) -> Stage {
    Stage {
        components: components
            .iter()
            .map(|c| ComponentId((*c).into()))
            .collect(),
        propellant_kg: propellant,
        engine: engine.map(|e| ComponentId(e.into())),
    }
}

/// A design with the given class, stages, mission and redundancy.
pub fn design(
    class: &str,
    stages: Vec<Stage>,
    mission: MissionReqs,
    redundancy: Vec<RedundancyBlock>,
) -> VehicleDesign {
    VehicleDesign {
        id: DesignId(0),
        faction: FactionId(0),
        class: class.into(),
        stages,
        redundancy,
        mission,
        parent: None,
        production_count: 0,
    }
}

/// A snapshot over a single design with the given maturity map.
pub fn snapshot(
    cat: Catalogue,
    d: VehicleDesign,
    maturity: BTreeMap<String, ComponentMaturity>,
) -> DesignSnapshot {
    let mut designs = BTreeMap::new();
    designs.insert(d.id, d);
    DesignSnapshot::new(designs, cat, maturity)
}

/// Redundancy block from component ids.
pub fn redundancy(components: &[&str]) -> RedundancyBlock {
    RedundancyBlock {
        components: components
            .iter()
            .map(|c| ComponentId((*c).into()))
            .collect(),
    }
}
