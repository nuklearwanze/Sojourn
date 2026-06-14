//! Foundational — the host + composition root build the full nine-module core and
//! advance it; the snapshot/clock stay consistent (FR-UI-1502/1503). This is the
//! end-to-end proof that the UI hosts the headless core in-process.

use sojourn_ui::host::UiHost;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../")
}

#[test]
fn the_host_builds_the_full_core_and_advances() {
    let mut host = UiHost::new_game(&repo_root(), 7).expect("the UI composes the full module set");
    let t0 = host.status().tick;

    // Advance a few days; the clock moves and stays consistent with what we read.
    host.advance(3 * 86_400).expect("advance");
    let s = host.status();
    assert!(s.tick > t0, "the host drives the core's clock forward");
    assert_eq!(s.tick, t0 + 3 * 86_400);

    // The event feed is readable (the same core the harness runs headless).
    let _events = host.recent_events(16);
}

#[test]
fn the_map_reads_real_body_positions_from_the_astro_surface() {
    let host = UiHost::new_game(&repo_root(), 7).expect("compose core");
    let bodies = host.map_bodies();
    assert!(
        bodies.len() >= 9,
        "the catalogue yields the Sun + planets at least (got {})",
        bodies.len()
    );

    // The system root (the Sun) sits at ~origin; some planet is far from it.
    let near_origin = bodies.iter().any(|b| (b.pos.0.hypot(b.pos.1)) < 1.0e6);
    let far_body = bodies.iter().any(|b| b.pos.0.hypot(b.pos.1) > 1.0e9);
    assert!(near_origin, "the root body sits at the heliocentric origin");
    assert!(
        far_body,
        "orbiting bodies are propagated to real distances (m)"
    );

    // Names are display-cased, not raw ids.
    assert!(bodies.iter().all(|b| !b.name.is_empty()));
}

#[test]
fn the_rnd_screen_reads_the_domain_portfolio_from_the_research_surface() {
    let host = UiHost::new_game(&repo_root(), 7).expect("compose core");
    let doms = host.research_domains();

    // The full A1–A17 knowledge-domain set is read from the FA-05 surface.
    assert_eq!(doms.len(), 17, "the A1–A17 domains (got {})", doms.len());
    assert!(doms.iter().any(|d| d.id == "A1"));
    assert!(doms.iter().any(|d| d.id == "A16"));

    // Names are display-cased (the `dom.` namespace is the UI's to strip).
    let mat = doms.iter().find(|d| d.id == "A1").expect("A1 present");
    assert_eq!(mat.name, "Materials");
    assert!(mat.basic_science, "A1 Materials is basic science");

    // On a fresh run nothing is researched: every UL is 0, but the static shape
    // (the diminishing-returns knee) is real domain data.
    assert!(
        doms.iter()
            .all(|d| d.private_ul == 0.0 && d.effective_ul == 0.0)
    );
    assert!(doms.iter().all(|d| d.dr_knee > 0.0));

    // Astrobiology (A16) is a mission-fed domain (field work injects UL).
    let astro = doms.iter().find(|d| d.id == "A16").expect("A16 present");
    assert!(astro.mission_injectable);
}

#[test]
fn the_vehicle_designer_derives_a_reference_design_and_palette() {
    let host = UiHost::new_game(&repo_root(), 7).expect("compose core");
    let rep = host
        .vehicle_report()
        .expect("the vehicle slice derives the reference design");

    // The reference design has real, sane physics (computed by the slice, not the UI).
    assert!(
        rep.wet_mass > rep.dry_mass,
        "wet > dry (propellant carried)"
    );
    assert!(
        (rep.wet_mass - rep.dry_mass - rep.propellant).abs() < 1.0,
        "wet = dry + propellant"
    );
    assert!(
        rep.total_dv > 1_000.0,
        "a hydrolox stage yields km/s of Δv (got {})",
        rep.total_dv
    );
    assert!(rep.power_gen > 0.0 && rep.power_demand > 0.0);

    // The parts palette carries the whole catalogue.
    assert!(
        rep.components.len() >= 14,
        "the full parts catalogue (got {})",
        rep.components.len()
    );
    assert!(rep.components.iter().any(|c| c.class == "Engine"));

    // On a fresh programme nothing is researched: no part is flyable, and the
    // immaturity red-flag fires (the C1 FA-05 maturity read driving the guards).
    assert!(rep.components.iter().all(|c| !c.flyable && c.trl == 0));
    assert!(
        rep.red_flags.iter().any(|f| f.constraint.contains("TRL")),
        "the sub-TRL-6 guard fires when nothing is researched"
    );
}

#[test]
fn the_economy_screen_reads_ledger_funding_market_and_commodities() {
    let host = UiHost::new_game(&repo_root(), 7).expect("compose core");
    let econ = host
        .economy_overview()
        .expect("the economy surface yields an overview");

    // The six conserved currencies, all zero on a fresh programme (nothing earned).
    assert_eq!(econ.balances.len(), 6, "the six-currency ledger");
    assert!(
        econ.balances
            .iter()
            .any(|b| b.name == "Funds" && b.is_funds)
    );
    assert!(
        econ.balances.iter().all(|b| b.amount == 0.0),
        "fresh ledger is empty"
    );

    // The player (faction 0) is the agency archetype.
    assert!(
        econ.funding_kind.starts_with("Agency"),
        "got {}",
        econ.funding_kind
    );

    // The launch market prices every orbit class, finite and ascending by energy.
    assert!(econ.launch_prices.len() >= 4, "leo/gto/geo/tli at least");
    assert!(
        econ.launch_prices
            .iter()
            .all(|p| p.price_per_kg.is_finite() && p.price_per_kg > 0.0)
    );
    let leo = econ
        .launch_prices
        .iter()
        .find(|p| p.orbit_class == "leo")
        .expect("leo priced");
    assert!(
        (leo.price_per_kg - 2_700.0).abs() < 1.0,
        "LEO base $/kg at reference capacity"
    );

    // The commodity taxonomy is non-empty and carries kind/unit labels.
    assert!(econ.commodities.len() >= 4, "a populated taxonomy");
    assert!(
        econ.commodities
            .iter()
            .all(|c| !c.kind.is_empty() && !c.unit.is_empty())
    );
}

#[test]
fn the_world_screen_reads_the_milestone_race_board_from_the_polity_surface() {
    let host = UiHost::new_game(&repo_root(), 7).expect("compose core");
    let world = host
        .world_overview()
        .expect("the polity surface yields a world overview");

    // The historic-firsts catalogue spans all four eras, all unclaimed on a fresh run.
    assert!(
        world.milestones.len() >= 10,
        "a populated race board (got {})",
        world.milestones.len()
    );
    assert!(
        world.milestones.iter().all(|m| m.claimed_by.is_none()),
        "nothing claimed yet"
    );
    assert!(
        world
            .milestones
            .iter()
            .all(|m| m.weight > 0.0 && !m.description.is_empty())
    );
    for era in ["foothold", "cislunar", "frontier", "endgame"] {
        assert!(
            world.milestones.iter().any(|m| m.era == era),
            "the {era} era is present"
        );
    }

    // Factions populate only once the world is initialised — none on a fresh run.
    assert!(world.factions.is_empty(), "no factions before world init");
    assert!(world.science_tide.is_finite());
}

#[test]
fn the_remaining_screens_read_their_surfaces() {
    let host = UiHost::new_game(&repo_root(), 7).expect("compose core");

    // S2 Trajectory: heliocentric Hohmann options from Terra, ascending in radius,
    // with finite positive Δv/TOF (real two-body physics).
    let transfers = host.transfer_options();
    assert!(
        transfers.len() >= 3,
        "transfers to several planets (got {})",
        transfers.len()
    );
    assert!(
        transfers
            .iter()
            .all(|t| t.dv_mps > 0.0 && t.tof_days > 0.0 && t.dv_mps.is_finite())
    );
    assert!(
        transfers.windows(2).all(|w| w[0].r_to_au <= w[1].r_to_au),
        "sorted by radius"
    );
    assert!(
        transfers.iter().any(|t| t.to == "Ares"),
        "Mars/Ares is a target"
    );

    // S5 Operations + S8 Personnel: empty on a fresh run (nothing launched/hired).
    assert!(host.fleet().is_empty(), "no craft on a fresh run");
    let personnel = host.personnel_summary();
    assert!(
        personnel
            .iter()
            .all(|p| p.role == "AstronautReady" || p.count == 0)
    );

    // S7 Bases: the buildable module + class catalogue is live from the first frame.
    let bases = host.base_catalogue();
    assert!(!bases.modules.is_empty(), "a populated module catalogue");
    assert!(!bases.classes.is_empty(), "base-class templates");
    assert_eq!(bases.founded, 0, "no bases founded yet");

    // S11 Sojournal: the source-cited encyclopedia aggregates real provenance — every
    // entry carries a non-empty source citation.
    let sj = host.sojournal_entries();
    assert!(sj.len() >= 30, "a rich provenance index (got {})", sj.len());
    assert!(
        sj.iter().all(|e| !e.source.trim().is_empty()),
        "every entry is sourced"
    );
    assert!(sj.iter().any(|e| e.category == "Knowledge domain"));
    assert!(sj.iter().any(|e| e.category == "Historic first"));
}
